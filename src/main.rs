#![windows_subsystem = "console"]
#![crate_type = "bin"]
mod bybit_exchange;
mod db;
mod misc;
mod utils;
use bybit::{http};
use bybit_exchange::{bybit_inter};
//use ftx::{options::Options, rest::*};
use rust_decimal::prelude::*;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::env;
use std::fs;
use terminal_size::{terminal_size, Height, Width};

use db::{get_db_info, history_location};

static VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Config {
	pub bybit_default_pair: String,
	pub bybit_default_sub: String,
	pub bybit_pub_key: String,
	pub bybit_priv_key: String,
	pub ratio_warn_num: f64,
}

#[tokio::main]
async fn main() {
	//creates termcrypt data folder & history subfolder if does not exist
	let _x = fs::create_dir_all(history_location().as_str());
	match _x {
		Ok(_x) => _x,
		Err(_e) => (),
	}

	//initiates database
	let mut db_info = get_db_info(true).await.unwrap();

	//default variables
	let mut pair: String = db_info.bybit_default_pair.to_string();
	let mut subaccount: String = db_info.bybit_default_sub.to_string();

	let mut bybit_api = http::Client::new(
		http::MAINNET_BYBIT,
		&db_info.bybit_pub_key,
		&db_info.bybit_priv_key,
	)
	.unwrap();

	//starts websockets
	//tokio::spawn(ftx_ws::ftx_websocket(opts));

	//gets terminal size
	let size = terminal_size();
	let mut terminal_is_wide = true;

	//wide if width is more than 70 characters
	if let Some((Width(width), Height(_h))) = size {
		if width < 70 {
			terminal_is_wide = false;
		}
	} else {
		terminal_is_wide = false
	}

	//outputs version and ascii art
	print!("{}[2J", 27 as char);
	if terminal_is_wide {
		utils::wideversion();
	} else {
		utils::slimversion();
	};
	println!();

	let line_main_location = format!("{}main.txt", history_location().as_str());
	if !std::path::Path::new(&line_main_location.to_string()).exists() {
		std::fs::File::create(line_main_location.to_string()).unwrap();
	}
	let mut line_main = Editor::<()>::new();
	line_main.load_history(&line_main_location).unwrap();

	let mut loop_iteration: i32 = 1;

	loop {
		//Start of loop
		//Takes input from user through terminal-like interface*/
		let mut is_real_command = false;
		let read_line =
			line_main.readline(format!("[{}]({})> ", subaccount.as_str(), pair.as_str()).as_str());

		match read_line {
			Ok(read_line) => {
				//add command to command history
				line_main.add_history_entry(read_line.as_str());
				
				//command handling
				match bybit_inter::handle_commands(bybit_inter::CommandHandling{
					//make this a struct one day
					command_input: read_line.as_str(),
					current_sub_account: &mut subaccount,
					current_pair: &mut pair,
					bybit_api: &mut bybit_api,
					//&mut q_account,
					_terminal_is_wide: &mut terminal_is_wide,
					database_info: &mut db_info,
				})
				.await
				{
					Ok(x) => {
						if !is_real_command && x {
							is_real_command = true
						}
					}
					Err(e) => {
						println!();
						eprintln!("!! Function Exited: {:?} !!", e);
						println!();
						continue;
					}
				};

				//miscellaneous command handling
				match misc::handle_commands(
					//make this a struct one day
					read_line.as_str(),
					&mut terminal_is_wide,
					loop_iteration,
					//&mut db_info,
				)
				.await
				{
					Ok(x) => {
						if x {
							is_real_command = true
						}
					}
					Err(e) => {
						println!();
						eprintln!("!! Function Exited: {:?} !!", e);
						println!();
						continue;
					}
				}

				//adds padding
				println!();
			}
			Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
				println!("Exiting...");
				println!();
				println!("{}", utils::boldt("Thank you for using termcrypt ;)"));
				break;
			}
			Err(e) => {
				println!();
				eprintln!("!! Something bad happened, be scared: {:?} !!", e);
				println!();
				break;
			}
		}
		if is_real_command {
			line_main.append_history(&line_main_location).unwrap();
		}
		loop_iteration += 1;
	}
}
