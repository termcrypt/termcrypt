#![windows_subsystem = "console"]
#![crate_type = "bin"]
use bybit::http::{self, Client as BybitClient};
use anyhow::{Error as AnyHowError, Result as AnyHowResult};
use serde::{Deserialize, Serialize};
use std::{io::{self}, env, fs, time::{Duration}};
use tui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
	cursor::DisableBlinking,
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

mod bybit_core;
mod db;
mod misc_commands;
mod sync;
mod utils;
mod ws;
mod userspace;
mod command_handling;
mod orders;

use bybit_core::{bybit_commands::{self, BybitStruct}};
use db::{get_db_info, history_location};
use sync::*;
use utils::{termbug, terminal_width};
//use command_handling;
use userspace::EventLogType;

// Cargo information
static VERSION: &str = env!("CARGO_PKG_VERSION");
static REPO: &str = "https://github.com/termcrypt/termcrypt";

// Values returned from initial database query
#[derive(Clone, Debug)]
pub struct Config {
	// Default user pair for Bybit
	pub bybit_default_pair: String,
	// Default user sub-account for Bybit
	pub bybit_default_sub: String,
	// Public API key for Bybit
	pub bybit_pub_key: String,
	// Private API key for Bybit
	pub bybit_priv_key: String,
	// User config ratio warn number;
	// Warns user if trade ratio is too low
	pub ratio_warn_num: f64,
}

// Available exchanges; bool inside is for if that exchange is enabled (set up) or not
#[derive(Serialize, Deserialize, Debug)]
pub enum ActiveExchanges {
	Bybit(bool),
	Ftx(bool),
}

// UI display arrangements
enum UIModes {
	// Events section hidden
	InputOnly,
	// Default UI (both events and commands)
	Split,
	// Whole UI is for events
	EventsOnly
}

// User interface location for function focus
pub enum UILocation {
	// Main output stream
	Main,
	// Background output stream (fills, etc)
	Events
}

// Userspace state
pub struct UserSpace {
	// Database information
	pub db_info: Config,
	// Current pair name
	pub pair: String,
	// Current sub-account name
	pub sub_account: String,
	// Bybit API 
	pub bybit_api: BybitClient,
	// Boolean for if the terminal is wide enough for ascii
	pub desktop_terminal: bool,
	// Current user input before submission
	pub input: String,
	// Old input before wiping linked box
	pub input_old: String,
	// Prefix preceding user input
	pub input_prefix: String,
    // History of user commands
    pub command_history: Vec<String>,
	// Scroll overflow vector for past commands out of sight
	pub command_history_scroll_overflow: Vec<String>,
	// Count of user commands
	pub command_count: u32,
	// Events (orders etc) history
	pub event_history: Vec<(String, EventLogType)>,
	// Tick rate of UI
	pub tick_rate: Duration,
	// Stream difference
	pub stream_differ: u16,
}

// Where the program starts :)
#[tokio::main]
async fn main() -> AnyHowResult<(), AnyHowError> {
	/* Use these args in the future
	let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
	loop {}
	*/

	
	
	// Creates termcrypt data folder & history subfolder if does not exist
	let _x = fs::create_dir_all(history_location().as_str()).unwrap_or_else(|_| {
		panic!("COULD NOT CREATE HISTORY DIRECTORY")
	});

	// Get terminal width
	let mut desktop_terminal = false;
	if terminal_width() >= 68 {
		desktop_terminal = true;
	}

	// Initiates database
	let db_info = get_db_info(true).await?;
	let mut command_history = Vec::new();

	// Start bybit api instance
	let mut bybit_api = http::Client::new(
		http::MAINNET_BYBIT,
		&db_info.bybit_pub_key,
		&db_info.bybit_priv_key,
	).unwrap();

	// Synchronize missed executions when app was not running
	match startup_sync(Some(&mut bybit_api)).await {
		Ok(_x) => { /*:L*/ }
		Err(_e) => {
			command_history.insert(0, format!("Failed to sync trades with ws, but termcrypt kept running: {_e:?}"));
		}
	};

	// Initiates userspace (UI and main loop)
	let mut userspace = UserSpace {
		pair: db_info.bybit_default_pair.to_owned(),
		sub_account: db_info.bybit_default_sub.to_owned(),
		input_prefix: format!("[{}]({})>", db_info.bybit_default_sub, db_info.bybit_default_pair),
		db_info,
		// Initiates Bybit API (note: make option on multi exchanges)
		bybit_api,
		desktop_terminal,
        input: String::new(),
		command_history,
		command_history_scroll_overflow: Vec::new(),
		command_count: 0,
		event_history: vec![
			("You liquidated everything".to_string(), EventLogType::Warning),
			("at 30202$ (BTC)".to_string(), EventLogType::SlFill),
			("at 31902$ (BTC)".to_string(), EventLogType::EntryFill),
			("at 2069$ (ETH)".to_string(), EventLogType::TpFill),
			
		],
		tick_rate: Duration::from_millis(50),
		stream_differ: 0,
		input_old: String::new(),
	};

	// Initiate terminal layout
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
		stdout,
		EnterAlternateScreen,
		EnableMouseCapture,
		DisableBlinking,
		terminal::SetTitle(format!("termcrypt v{VERSION}"))
	)?;

	// Uses crossterm backend for terminal function reliability
    let backend = CrosstermBackend::new(stdout);
	// Create the tui terminal instance using crossterm
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it (main app)
    userspace.run_app(&mut terminal).await?;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

	// End credits
	println!();
	println!("{}", utils::boldt("Thank you for using termcrypt :)"));

	Ok(())
}