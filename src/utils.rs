use ansi_term::ANSIGenericString;
use ansi_term::Style;
use anyhow::{bail, Error as AnyHowError};
use rust_decimal::{Decimal, RoundingStrategy};
use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::{
	UserSpace,
	UILocation::*,
	db::history_location
};

// User query while in command
pub fn askout(prefix: &str, file_name: Option<String>) -> Result<String, AnyHowError> {
	let mut line_genask = Editor::<()>::new();
	let mut line_current_location: String = "".to_string();

	if let Some(ref file_name_ac) = file_name {
		line_current_location = format!(
			"{}{}.txt",
			history_location().as_str(),
			file_name_ac.as_str()
		);

		// Creates history directory and file if not made already
		if !std::path::Path::new(&line_current_location).exists() {
			std::fs::File::create(&line_current_location).unwrap();
		}
		line_genask.load_history(&line_current_location).unwrap();
	}

	let readline = line_genask.readline(format!("  {}>> ", prefix).as_str());

	match readline {
		// Add some smart error handling to re-loop from askout function
		Ok(readline) => {
			// If user is using askout readline and enters q, app will exit
			if readline == *"q" {
				bail!("User stopped");
			}

			if let Some(_x) = file_name {
				line_genask.add_history_entry(readline.as_str());
				line_genask.append_history(&line_current_location).unwrap();
			}
			Ok(readline)
			
		}
		Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
			println!("Exiting...");
			panic!();
		}
		Err(e) => {
			bail!("!! Cannot Readline: {e:?} !!");
		}
	}
}

// Yes/No prompt
pub fn yn(text: String) -> Result<(), AnyHowError> {
	match text.to_uppercase().as_str() {
		"Y" | "YES" => {
			println!("  HIT: Confirmed");
		}
		"N" | "NO" => {
			println!("  HIT: User denied function")
		}
		_ => {
			bail!("Did not understand input")
		}
	}
	Ok(())
}

// Returns text styled bold
pub fn boldt(text: &str) -> ANSIGenericString<'_, str> {
	Style::new().bold().paint(text)
}

// Rounds up away from zero
pub fn _round_dp_up(num: Decimal, places: u32) -> Decimal {
	num.round_dp_with_strategy(places, RoundingStrategy::MidpointAwayFromZero)
}

// Rounds down to zero
pub fn round_dp_tz(num: Decimal, places: u32) -> Decimal {
	num.round_dp_with_strategy(places, RoundingStrategy::ToZero)
}

// Returns "s" if the number is bigger than 1 (used for outputs to user)
pub fn s_or_not(num: usize) -> String {
	if num > 1 {
		return "s".to_string()
	}
	"".to_string()
}

// Wide ascii art
/*
pub fn wideversion(us: &mut UserSpace) {
	let ascii: [String; 7] = [
		"".to_string(),
		"  _______ _______ ______ _______ ______ ______ ___ ___ ______ _______ ".to_string(),
		" |_     _|    ___|   __ ⑊   |   |      |   __ ⑊   |   |   __ ⑊_     _|".to_string(),
		"   |   | |    ___|      <       |   ---|      <⑊     /|    __/ |   |  ".to_string(),
		"   |___| |_______|___|__|__|_|__|______|___|__| |___| |___|    |___|  ".to_string(),
		"".to_string(),
		format!(
			"  v{} License: 🟢 AGPL3+                            {}",
			crate::VERSION,
			chrono::Local::now().format("[Now] %H:%M:%S")
		)
	];

	for item in ascii {
		us.prnt(item)
	}
}
*/

// Output ascii art to the user
pub fn output_ascii(us: &mut UserSpace) {
	let ascii: [String; 5] = [
		String::new(),
		" <termcrypt>".to_string(),
		String::new(),
		format!("  v{} License: 🟢 AGPL3+", crate::VERSION),
		String::new(),
	];

	for item in ascii {
		us.prnt(item)
	}
}

// Shortcut for printing blank line
pub fn bl(us: &mut UserSpace) {
	us.prnt(String::new());
}

// Shortcut for printing multiple blank lines
pub fn _mbl(count: i32) {
	let mut i = 0;
	while i < count {
		println!();
		i += 1
	}
}

// Module for formatting debugging and errors
pub mod termbug {
	use core::fmt::Debug;
	use crate::UserSpace;

	pub fn error<T: Debug>(err_msg: T, us: &mut UserSpace) {
		let err_to_check = format!("{:?}", err_msg).to_lowercase();
		let mut _error = vec![format!("!-- Error: {err_msg:?} --!")];

		// Make user mid-command bails look cleaner
		_error = if err_to_check.contains("user quit") {
			vec!["-- You quit the current command --".to_string()]
		} else {
			vec![
				String::new(),
				format!("!-- Error: {err_msg:?} --!"),
				format!("!-- If needed, report this error to {}. For reference, you are using version {} --!", crate::REPO, crate::VERSION),
			]
		};

		for line in _error {
			us.prnt(line);
		}
	}
	
	// Debug function goes here?
}