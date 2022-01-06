use anyhow::{bail, Error, Result};
use chrono::{DateTime, Duration, NaiveTime, TimeZone, Utc};
use rust_decimal::prelude::*;
use super::db::*;
use rust_decimal_macros::dec;
use polodb_core::Database;
use terminal_size::{terminal_size, Height, Width};

use super::utils::{
	boldt,
	//round_dp_up,
	round_dp_tz,
	slimversion,
	wideversion,
	bl,
	askout as ask
};

//use super::utils::boldt as boldt;

//Command Handling
pub async fn handle_commands(x: &str, wide: &mut bool, loop_iteration: i32) -> Result<bool, Error> {
	let mut isrealcommand = true;
	match x {
		//lists all commands
		"h" | "help" => {
			//i would have made this dynamic but brain shite
			println!("{}", boldt("UTILITY"));
			println!("  clr | clear - clear the terminal");
			println!("  h | help - get information about commands");
			println!("  q - quits input when inside function");
			println!("  date - get current local and utc date");
			println!("  time - get current local and utc time");
			println!("  ses | sessions - get trading session times");
			//println!("{}", boldt("SUBACCOUNTS"));
			//println!("  subs - list all subaccounts");
			//println!("  sub [nickname] - change subaccount (case sensitive)");
			//println!("  bal | balance - get balances of subaccount");

			println!("{}", boldt("MARKETS"));
			println!("  search | search [query] - return all pairs containing query");
			println!("  p | price - return the mark, ask and bid price for current pair");
			println!("  pair | pair [query] - lets you change the current pair");
			//println!("  ob | orderbook - get display of orderbook");

			//println!("{}", boldt("ORDERS"));
			//println!("  lev - get current account leverage");
			//println!("  lev [number] - change leverage to chosen number");
			//println!("  o | order - start an order");

			println!("{}", boldt("SETTINGS"));
			//println!("  def | defaults - change termcrypt startup defaults");
			println!("  conf | config - change termcrypt configuration variables");

			println!("{}", boldt("KEYBINDS"));
			println!("  [UP ARROW] - Replaces input with previous command");
			println!("  [DOWN ARROW] - Replaces input with the latter command");
			println!();
			println!("  More information is available in our documentation.");
		},
		"about" => {
			bl();
			println!("{}", boldt("About termcrypt:"));
			println!("termcrypt is a project to bring maximum efficiency to crypto trading.");
			println!("It is an open sourced app licensed under AGPL3+ (a copyleft license).");
			println!("You can visit our repository at https://github.com/termcrypt/termcrypt");
			bl();
			println!("You are running termcrypt version: {}", super::VERSION);
		},
		"clr" | "clear" => print!("{}[2J", 27 as char),
		"conf" | "config" => {
			let mut database = Database::open(database_location().as_str()).unwrap();

			println!("  1. Change SLTP-Ratio warning number");
			let choice = ask("[Option Number]", Some("defaultsoptionnumber".to_string()))?;

			match choice.as_str() {
				"1" => {
					println!();
					let new_ratio = ask("  [New ratio warning number]", None)?.parse::<Decimal>()?;
					db_insert_config(&mut database, "ratio_warn_num", &new_ratio.to_string())?;
					println!("  Changed ratio warning number successfully");
				}
				_ => {
					println!("  {}", boldt("!! Not a choice !!"));
				}
			}
		},
		"ses" | "sessions" => {
			//trading sessions time information
			println!("{}", boldt("Trading Sessions"));
			let utc_time = Utc::now().time();
			let utc_now = Utc::now();

			let mut circlecolor: &str;
			let mut timetoevent: String;
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

			//NY SESSION

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
			timetoevent = format!(
				"{} in: {}h {}m",
				if isopen { "Closes" } else { "Opens" },
				round_dp_tz(times.hourstoevent, 0),
				round_dp_tz(times.minutestoevent, 0) /*, round_dp_tz(secondstoevent, 0)*/
			);

			println!("  {} NY (Optimal)", circlecolor);
			println!("    {}", timetoevent);
			println!("    (12AM-8PM UTC)");
			println!();

			//ASIA SESSION

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

			println!("  {} ASIA (Optimal)", circlecolor);
			println!("    {}", timetoevent);
			println!("    (11PM-4AM UTC)");
		}
		"date" => {
			println!("  {}", chrono::Utc::now().format("[UTC] %b %-d %Y"));
			println!("  {}", chrono::Local::now().format("[Now] %b %-d %Y"));
		}
		"time" => {
			println!("  {}", chrono::Utc::now().format("[UTC] %H:%M:%S"));
			println!("  {}", chrono::Local::now().format("[Now] %H:%M:%S"));
		}
		"afetch"|"aclear" => {
			if x == "aclear" {print!("{}[2J", 27 as char)}
			//gets terminal size
			let size = terminal_size();

			//wide if width is more than 70 characters
			if let Some((Width(width), Height(_h))) = size {
				if width < 70 {
					*wide = false;
				} else {
					*wide = true
				}

				if wide.to_owned() {
					wideversion();
				} else {
					slimversion();
				};
			}
		}
		"ping" => {
			println!("{}", boldt("🏓 Pong!"));
		}
		"count" => {
			println!("{} Commands this session", loop_iteration);
		}
		_ => (isrealcommand = false),
	}
	Ok(isrealcommand)
}

//Calculations
#[derive(Debug)]
pub struct OrderCalcEntry {
	pub total_liquid: f64,
	pub risk: f64,
	pub stoploss: f64,
	pub takeprofit: f64,
	pub entry: f64,
}

#[derive(Debug)]
pub struct OrderCalcExit {
	pub quantity: f64,
	pub islong: bool,
	pub tpslratio: f64,
}

pub fn calculate_order(ov: OrderCalcEntry) -> Result<OrderCalcExit, Error> {
	let error_margin = f64::EPSILON;

	let mut islong = true;
	let mut quantity: f64 = 1.0;
	let mut tpslratio: f64 = 1.0;

	//Checks if parameters are correct
	//More should be added here for more cases

	if (ov.risk > 100.0)
		||(ov.stoploss - ov.takeprofit).abs() < error_margin
		||(ov.stoploss - ov.entry).abs() < error_margin
		||(ov.takeprofit - ov.entry).abs() < error_margin
		|| ov.risk < 0.1
	{
		bail!("Invalid parameters (error in calculating order size) - Check your inputs");
	}

	//Checks for Long
	if ov.stoploss < ov.takeprofit && ov.entry > ov.stoploss && ov.entry < ov.takeprofit {
		islong = true;
		quantity = ov.total_liquid * (ov.risk / ((1.0 - (ov.stoploss / ov.entry)) * 100.0));
		tpslratio = (ov.takeprofit - ov.entry) / (ov.entry - ov.stoploss); //toFixed(2)
	}
	//Cheks for Short
	else if ov.stoploss > ov.takeprofit && ov.entry < ov.stoploss && ov.entry > ov.takeprofit {
		islong = false;
		quantity = ov.total_liquid * (ov.risk / ((1.0 - (ov.entry / ov.stoploss)) * 100.0));
		tpslratio = (ov.entry - ov.takeprofit) / (ov.stoploss - ov.entry); //toFixed(2)
	}

	Ok(OrderCalcExit {
		quantity,
		islong,
		tpslratio,
	})
}
