#![windows_subsystem = "console"]
#![crate_type = "bin"]
mod db;
mod ftx_exchange;
mod misc;
mod utils;
use ftx::{options::Options, rest::*};
use ftx_exchange::*;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::env;
use std::fs;
use terminal_size::{terminal_size, Height, Width};

use db::{get_db_info, history_location};

static VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Config {
	pub default_pair: String,
	pub default_sub: String,
	pub ftx_pub_key: String,
	pub ftx_priv_key: String,
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
	let mut pair: String = db_info.default_pair.to_string();
	let mut subaccount: String = db_info.default_sub.to_string();

	//add check for valid keys later
	let opts = Options {
		key: Some(db_info.ftx_pub_key.to_owned()),
		secret: Some(db_info.ftx_priv_key.to_owned()),
		subaccount: if Some(subaccount.to_string()) == Some("def".to_string()) {
			None
		} else {
			Some(subaccount.to_string())
		},
		endpoint: ftx::options::Endpoint::Com,
	};
	let mut api = Rest::new(opts.to_owned());

	//gets user account object
	let mut q_account = api.request(GetAccount).await.unwrap();

	//starts websockets
	tokio::spawn(ftx_ws::ftx_websocket(opts));

	//gets terminal size
	let size = terminal_size();
	let mut wide = true;

	//wide if width is more than 70 characters
	if let Some((Width(width), Height(_h))) = size {
		if width < 70 {
			wide = false;
		}
	} else {
		wide = false
	}

	//outputs version and ascii art
	if wide {
		utils::wideversion();
	} else {
		utils::slimversion();
	};
	println!();
	//let loc = history_location();
	//let root = Path::new(&loc);
	//env::set_current_dir(&root);

	let line_main_location = format!("{}main.txt", history_location().as_str());

	if !std::path::Path::new(&line_main_location.to_string()).exists() {
		std::fs::File::create(line_main_location.to_string()).unwrap();
	}

	let mut line_main = Editor::<()>::new();

	line_main.load_history(&line_main_location).unwrap();

	//println!("{}", line_main_location.as_str());

	let mut loop_iteration: i32 = 1;

	loop {
		//Start of loop
		//Takes input from user through terminal-like interface*/
		let mut isrealcommand = false;
		let read_line =
			line_main.readline(format!("[{}]({})> ", subaccount.as_str(), pair.as_str()).as_str());

		match read_line {
			Ok(read_line) => {
				line_main.add_history_entry(read_line.as_str());
				//ftx command handling
				match ftx_inter::handle_commands(
					//make this a struct one day lazy ass
					read_line.as_str(),
					&mut subaccount,
					&mut pair,
					&mut api,
					&mut q_account,
					wide,
					&mut db_info,
				)
				.await
				{
					Ok(x) => {
						if !isrealcommand && x {
							isrealcommand = true
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
					//make this a struct one day lazy ass
					read_line.as_str(),
					wide,
					loop_iteration,
					//&mut db_info,
				)
				.await
				{
					Ok(x) => {
						if !isrealcommand && x {
							isrealcommand = true
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
		if isrealcommand {
			line_main.append_history(&line_main_location).unwrap();
		}
		loop_iteration += 1;
	}
}
