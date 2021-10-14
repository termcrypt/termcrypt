use ansi_term::ANSIGenericString;
use ansi_term::Style;
use anyhow::{bail, Error};
use rust_decimal::{Decimal, RoundingStrategy};
use rustyline::error::ReadlineError;
use rustyline::Editor;

use super::db::history_location;

pub fn askout(prefix: &str, file_name: Option<String>) -> Result<String, Error> {
	let mut line_genask = Editor::<()>::new();
	if let Some(ref file_name_ac) = file_name {
		let line_current_location = format!(
			"{}{}.txt",
			history_location().as_str(),
			file_name_ac.as_str()
		);
		//creates history directory and file if not made already
		if !std::path::Path::new(&line_current_location.to_string()).exists() {
			std::fs::File::create(line_current_location.to_string()).unwrap();
			println!("bababooey")
		}

		line_genask.load_history(&line_current_location).unwrap();
	}

	let readline = line_genask.readline(format!("  {}>> ", prefix).as_str());

	match readline {
		//add some smart error handling to re-loop from askout function
		Ok(readline) => {
			//if user is using askout readline and enters q, app will exit
			if readline == *"q" {
				bail!("User stopped");
			} else {
				if let Some(ref file_name_ac) = file_name {
					let line_current_location = format!(
						"{}{}.txt",
						history_location().as_str(),
						file_name_ac.as_str()
					);
					line_genask.append_history(&line_current_location).unwrap();
					println!("bruh");
				}
				Ok(readline)
			}
		}
		Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
			println!("Exiting...");
			println!();
			println!("{}", boldt("Thank you for using termcrypt ;)"));
			println!();
			panic!();
		}
		Err(e) => {
			panic!("{}", e);
		}
	}
}

pub fn yn(text: String) -> Result<(), Error> {
	match text.as_str() {
		"y" | "yes" | "Y" | "YES" => {
			println!("  HIT: Confirmed");
			Ok(())
		}
		_ => {
			bail!("User stopped")
		}
	}
}

pub fn boldt(text: &str) -> ANSIGenericString<'_, str> {
	Style::new().bold().paint(text)
}

pub fn ftx_formattedpair(pairr: [&str; 2]) -> String {
	let divider: &str;
	match pairr[1].to_uppercase().as_str() {
		"PERP" | "1231" => divider = "-",
		_ => divider = "/",
	}
	[
		pairr[0].to_uppercase(),
		divider.to_string(),
		pairr[1].to_uppercase(),
	]
	.concat()
}

pub fn ftx_getsuffixsymbol(pair: &str) -> &str {
	let usdsigns: [&str; 3] = ["USD", "PERP", "USDT"];
	for item in &usdsigns {
		if pair.ends_with(item) {
			return "$";
		}
	}
	match pair {
		x if x.ends_with("BTC") => "₿",
		x if x.ends_with("ETH") => "₿",
		_ => "",
	}
}

pub fn _round_dp_up(num: Decimal, places: u32) -> Decimal {
	num.round_dp_with_strategy(places, RoundingStrategy::MidpointAwayFromZero)
}

pub fn round_dp_tz(num: Decimal, places: u32) -> Decimal {
	num.round_dp_with_strategy(places, RoundingStrategy::ToZero)
}

pub fn _sideret(text: &str) {
	println!();
	println!("{}", boldt(text));
	println!("Continue your previous location ⌄ below ⌄");
}

pub fn wideversion() {
	print!("{}[2J", 27 as char);
	println!();
	println!("  _______ _______ ______ _______ ______ ______ ___ ___ ______ _______ ");
	println!(" |_     _|    ___|   __ ⑊   |   |      |   __ ⑊   |   |   __ ⑊_     _|");
	println!("   |   | |    ___|      <       |   ---|      <⑊     /|    __/ |   |  ");
	println!("   |___| |_______|___|__|__|_|__|______|___|__| |___| |___|    |___|  ");
	println!();
	println!("  v{}. License: 🟢 AGPL3+", super::VERSION);
}

pub fn slimversion() {
	print!("{}[2J", 27 as char);
	println!();
	println!("  {}", boldt("<termcrypt>"));
	println!();
	println!("  v{}. License: 🟢 AGPL3+", super::VERSION);
}
