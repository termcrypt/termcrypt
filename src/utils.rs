use ansi_term::ANSIGenericString;
use ansi_term::Style;
use anyhow::{bail, Error, Result};
use rust_decimal::{Decimal, RoundingStrategy};
use terminal_size::{terminal_size, Height, Width};

use crate::{UserSpace, db::history_location};

// Yes/No prompt
pub fn yn(text: String) -> Result<(), Error> {
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
	let ascii: [String; 4] = [
		String::new(),
		" <termcrypt>".to_string(),
		String::new(),
		format!("  v{} License: 🟢 AGPL3+", crate::VERSION)
	];

	for item in ascii {
		us.prnt(item)
	}
}

// Splits a string into a vector of strings to appeal to a width (used for word wrap)
pub fn sub_strings(string: String, split_len: usize) -> Vec<String> {
    let mut subs: Vec<String> = Vec::with_capacity(string.len() / split_len);
    let mut iter = string.chars();
    let mut pos = 0;

	// Case if "" is passed
	if string.is_empty() {
		return vec!["".to_string()]
	};

    while pos < string.len() {
        let mut len = 0;
        for ch in iter.by_ref().take(split_len) {
            len += ch.len_utf8();
        }
        subs.insert(0, (&string[pos..pos + len]).to_string());
        pos += len;
    }
    subs
}

// Gets terminal width
pub fn terminal_width() -> u16 {
	// Get terminal width
	let size = terminal_size();

	// Checks if UI is big enough to contain wide ascii (removed?)
	if let Some((Width(width), Height(_h))) = size {
		width
	} else {
		0
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
		} else if err_to_check.contains("keys") {
			vec![
				String::new(),
				"!-- Command needs exchange API keys to function --!".to_string(),
				format!("!-- Run the 'keys' command to set up API keys for {} --!", us.active_exchange),
			]
		} else if 
		err_to_check.contains("network is unreachable") ||
		err_to_check.contains("failed to lookup address") {
			vec![
				String::new(),
				"?-- Network cannot be reached: do you have internet --?".to_string(),
			]
		} else {
			vec![
				String::new(),
				format!("!-- Error: {err_msg:?}"),
				format!("If needed, report this error to {}.\nFor reference, you are using version {} --!", crate::REPO, crate::VERSION),
			]
		};

		for line in _error {
			us.prnt(line);
		}
		us.bl();
	}
	
	// Debug function goes here?
}