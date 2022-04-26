use crate::{db::*, UILocation::*};
use anyhow::{bail, Error as AnyHowError, Result};
use chrono::{DateTime, Duration, NaiveTime, TimeZone, Utc};
use polodb_core::Database;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use terminal_size::{terminal_size, Height, Width};

use crate::utils::{
	askout as ask,
	bl,
	boldt,
	round_dp_tz,
	output_ascii,
	s_or_not
};

//use super::utils::boldt as boldt;

// Command Handling
pub async fn handle_commands(us: &mut crate::UserSpace) -> Result<bool, AnyHowError> {
	let mut isrealcommand = true;
	let x = us.input_old.as_str();
	match x {
		// Lists all commands
		"h" | "help" => {
			// I would have made this dynamic but brain shite
			let help_msg = [
				"UTILITY",
				" clr | clear - clear the terminal",
				" h | help - get information about commands",
				" q - quits input when inside function",
				" date - get current local and utc date",
				" time - get current local and utc time",
				" ses | sessions - get trading session times",
				"MARKETS",
				" search | search [query] - query pairs",
				" p | price - return mark, ask and bid price for pair",
				" pair | pair [query] - lets you change the current pair",
				"SETTINGS",
				" conf | config - change termcrypt configuration variables",
				"KEYBINDS",
				" [UP ARROW] - Replaces input with previous command",
				" [DOWN ARROW] - Replaces input with the latter command",
				"",
				" More information is available in our documentation.",
			];

			for line in help_msg {
				us.prnt(line.to_string());
			}

			//println!("{}", boldt("SUBACCOUNTS"));
			//println!("  subs - list all subaccounts");
			//println!("  sub [nickname] - change subaccount (case sensitive)");
			//println!("  bal | balance - get balances of subaccount");
			//println!("  ob | orderbook - get display of orderbook");
			//println!("{}", boldt("ORDERS"));
			//println!("  lev - get current account leverage");
			//println!("  lev [number] - change leverage to chosen number");
			//println!("  o | order - start an order");
			//println!("  def | defaults - change termcrypt startup defaults");
		}
		"about" => {
			let about = [
				String::new(),
				"About termcrypt".to_string(),
				" termcrypt is a project to bring maximum efficiency to crypto trading.".to_string(),
				" It is an open sourced app licensed under AGPL3+ (a copyleft license).".to_string(),
				" You can visit our repository at https://github.com/termcrypt/termcrypt".to_string(),
				String::new(),
				format!(" You are running termcrypt version: {}", super::VERSION)
			];

			for line in about {
				us.prnt(line);
			}
		}
		"clr" | "clear" => us.clear_commands(),
		"conf" | "config" => {
			let mut database = Database::open(database_location().as_str()).unwrap();

			println!("  1. Change SLTP-Ratio warning number");
			let choice = ask("[Option Number]", Some("defaultsoptionnumber".to_string()))?;

			match choice.as_str() {
				"1" => {
					println!();
					let new_ratio =
						ask("  [New ratio warning number]", None)?.parse::<Decimal>()?;
					db_insert_config(&mut database, "ratio_warn_num", &new_ratio.to_string())?;
					println!("  Changed ratio warning number successfully");
				}
				_ => {
					println!("  {}", boldt("!! Not a choice !!"));
				}
			}
		}
		"ses" | "sessions" => {
			// Trading session time information
			us.prnt("Trading Sessions".to_string());
			let utc_time = Utc::now().time();
			let utc_now = Utc::now();

			let mut circlecolor: &str;
			let mut eventtime;
			let mut isopen: bool;

			pub struct Times {
				hourstoevent: Decimal,
				minutestoevent: Decimal,
				_secondstoevent: Decimal,
			}

			fn times_until<Tz2: TimeZone>(eventtiming: DateTime<Tz2>) -> Times {
				let duration = eventtiming
					.signed_duration_since(Utc::now())
					.to_std()
					.unwrap();

				let hours = (Decimal::from_str(duration.as_secs().to_string().as_str()).unwrap()
					/ dec!(60)) / dec!(60);
				let minutes = (hours - round_dp_tz(hours, 0)) * dec!(60);
				let seconds = (minutes - round_dp_tz(minutes, 0)) * dec!(60);

				Times {
					hourstoevent: hours,
					minutestoevent: minutes,
					_secondstoevent: seconds,
				}
			}

			// NY Session

			if utc_time >= NaiveTime::from_hms(12, 0, 0) && utc_time < NaiveTime::from_hms(18, 0, 0)
			{
				circlecolor = "🟢";
				eventtime = utc_now.date().and_hms(18, 0, 0);
				isopen = true;
			} else {
				circlecolor = "🔴";
				isopen = false;

				if utc_time < NaiveTime::from_hms(12, 0, 0)
					&& utc_time >= NaiveTime::from_hms(0, 0, 0)
				{
					eventtime = utc_now.date().and_hms(12, 00, 0);
				} else {
					eventtime = (utc_now + Duration::days(1)).date().and_hms(12, 00, 0);
				}
			}

			let times = times_until(eventtime);
			let mut timetoevent = format!(
				"{} in: {}h {}m",
				if isopen { "Closes" } else { "Opens" },
				round_dp_tz(times.hourstoevent, 0),
				round_dp_tz(times.minutestoevent, 0) /*, round_dp_tz(secondstoevent, 0)*/
			);

			us.prnt(format!("  {} NY (Optimal)", circlecolor));
			us.prnt(format!("    {}", timetoevent));
			us.prnt("    (12AM-8PM UTC)".to_string());
			bl(us);

			// Asia Session

			if utc_time >= chrono::NaiveTime::from_hms(23, 0, 0)
				|| utc_time < chrono::NaiveTime::from_hms(4, 0, 0)
			{
				circlecolor = "🟢";
				isopen = true;

				if utc_time >= chrono::NaiveTime::from_hms(23, 0, 0)
					&& utc_time <= chrono::NaiveTime::from_hms(23, 59, 0)
				{
					eventtime = (utc_now + Duration::days(1)).date().and_hms(4, 0, 0);
				} else {
					eventtime = utc_now.date().and_hms(4, 0, 0);
				}
			} else {
				circlecolor = "🔴";
				eventtime = utc_now.date().and_hms(23, 0, 0);
				isopen = false;
			}

			let times = times_until(eventtime);
			timetoevent = format!(
				"{} in: {}h {}m",
				if isopen { "Closes" } else { "Opens" },
				round_dp_tz(times.hourstoevent, 0),
				round_dp_tz(times.minutestoevent, 0) /*, round_dp_tz(secondstoevent, 0)*/
			);

			us.prnt(format!("  {} ASIA (Optimal)", circlecolor));
			us.prnt(format!("    {}", timetoevent));
			us.prnt("    (11PM-4AM UTC)".to_string());
		}
		"date" => {
			us.prnt(format!("  {}", chrono::Utc::now().format("[UTC] %b %-d %Y")));
			us.prnt(format!("  {}", chrono::Local::now().format("[Now] %b %-d %Y")));
		}
		"time" => {
			us.prnt(format!("  {}", chrono::Utc::now().format("[UTC] %H:%M:%S")));
			us.prnt(format!("  {}", chrono::Local::now().format("[Now] %H:%M:%S")));
		}
		"afetch" | "aclear" => {
			// "aclear" prevents terminal from printing ""...> afetch" above output
			if x == "aclear" {
				us.clear_commands();
			}
			// Get terminal size
			let size = terminal_size();

			// Wide ascii if width is more than 70 characters (this is reused code lol)
			if let Some((Width(_width), Height(_h))) = size {
				/*
				if width < 70 {
					//let e = *us;
					us.desktop_terminal = false;
				} else {
					us.desktop_terminal = true
				}

				if us.desktop_terminal.to_owned() {
					wideversion(us);
				} else {
					slimversion(us);
				};
				*/
				output_ascii(us);
			}
		}
		"ping" => {
			us.prnt(String::from("🏓 Pong!"));
		}
		"count" => {
			us.prnt(format!("{} Command{} this session", us.command_count, s_or_not(us.command_count as usize)));
		}
		_ => (isrealcommand = false),
	}
	Ok(isrealcommand)
}

// Universal defaults for configuration
pub fn universal_defaults() {
	println!("  There are no available non-exchange defaults right now.");
}

// Input variables into order calculation function
#[derive(Debug)]
pub struct OrderCalcEntry {
	// Total available quantity
	pub total_liquid: f64,
	// Trade risk (on SL) in percentage
	pub risk: f64,
	// Trade stoploss price
	pub stoploss: f64,
	// Trade take-profit price
	pub takeprofit: f64,
	// Trade potential entry price
	pub entry: f64,
}

// Calculated variables exiting order calculation
#[derive(Debug)]
pub struct OrderCalcExit {
	// Order quantity accounting for risk and other factors
	pub quantity: f64,
	// Bool for if variable is of order type long or short
	pub islong: bool,
	// Take-profit/Stop-loss ratio (R rate)
	pub tpslratio: f64,
}

// User trade entry type (technical order type)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EntryType {
	// Market order type (accepting offer)
	Market,
	// Limit order type (creating offer)
	Limit,
}

// Exchange type for local trade struct
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Exchange {
	Ftx,
	Bybit,
}

// Exchange coin market type (exchange specific)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ExchangeContext {
	// Account requires coin itself to place order (such as BTC)
	// Supports leverage
	BybitInverse,
	// Account requires asset coin is paired with to palce order (such as USDT)
	// Supports leverage
	BybitLinear,
	// Does not support leverage
	_BybitSpot,
	// Derivatives that expire at specific dates
	// Supports leverage
	_BybitFutures,
}

// Application-individual order entry types (not technical order type)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OrderEntryType {
	// Immediate order
	Market,
	// Trigger order at a certain level
	Limit,
	// Not used at the moment...
	Conditional,
	// Orderbook order (opens a limit order at a user specified step difference from mark price)
	OrderBook,
	// More types to come...
}

// Direction type for local trade struct
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OrderDirection {
	// Profits when price goes up
	Long,
	// Profits when price goes down
	Short,
}

// Calculate order quantities and validate inputs
pub fn calculate_order(ov: OrderCalcEntry) -> Result<OrderCalcExit, AnyHowError> {
	let error_margin = f64::EPSILON;

	let mut _islong = true;
	let mut _quantity: f64 = 1.0;
	let mut _tpslratio: f64 = 1.0;

	// Checks if parameters are correct
	// More should be added here for more cases - sometimes invalid data can be passed!

	if (ov.risk > 100.0)
		// Check if inputs are not 0
		|| (ov.stoploss - ov.takeprofit).abs() < error_margin
		|| (ov.stoploss - ov.entry).abs() < error_margin
		|| (ov.takeprofit - ov.entry).abs() < error_margin
		|| ov.risk < 0.1
	{
		bail!("Invalid parameters (error in calculating order size) - Check your inputs");
	}

	// Checks if order is of long type or is of short type
	if ov.stoploss < ov.takeprofit && ov.entry > ov.stoploss && ov.entry < ov.takeprofit {
		_islong = true;
		_quantity = ov.total_liquid * (ov.risk / ((1.0 - (ov.stoploss / ov.entry)) * 100.0));
		_tpslratio = (ov.takeprofit - ov.entry) / (ov.entry - ov.stoploss); //toFixed(2)
	}
	else if ov.stoploss > ov.takeprofit && ov.entry < ov.stoploss && ov.entry > ov.takeprofit {
		_islong = false;
		_quantity = ov.total_liquid * (ov.risk / ((1.0 - (ov.entry / ov.stoploss)) * 100.0));
		_tpslratio = (ov.entry - ov.takeprofit) / (ov.stoploss - ov.entry); //toFixed(2)
	} else {
		bail!("Something bad happened in calculate_order!")
	}

	Ok(OrderCalcExit {
		quantity: _quantity,
		islong: _islong,
		tpslratio: _tpslratio,
	})
}
