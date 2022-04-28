use crate::db::*;
use anyhow::{bail, Error as AnyHowError, Result as AnyHowResult};
use chrono::{DateTime, Duration, NaiveTime, TimeZone, Utc};
use polodb_core::Database;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use tui::{
	backend::Backend,
    Terminal,
};

use crate::{
	UserSpace,
	utils::{
		round_dp_tz,
		output_ascii,
		s_or_not
	}
};

pub async fn help(us: &mut UserSpace) -> AnyHowResult<(), AnyHowError> {
	// Dynamically listing these would be "harder" somebody said. Oh well...
	let help_msg = [
		"UTILITY",
		" h | help - Get information about commands",
		" about | info - Information about the project",
		" afetch | aclear - Ascii art with info",
		" clr | clear - Clear the terminal",
		" date - Get current local and utc date",
		" time - Get current local and utc time",
		" ses | sessions - Get trading session times",
		"MARKETS",
		" bal | balance - Get balances of subaccount",
		" search | search [query] - Query pairs",
		" pair | pair [query] - Lets you change the current pair",
		" p | price - Return mark, ask and bid price for pair",
		"ORDERS",
		" order | o - Start an order",
		" m - Start a market order",
		" l - Start a limit order",
		" ob - Start an OB-based order",
		"SETTINGS",
		" def | defaults - Change startup defaults (exchange specific)",
		" conf | config - Change configuration settings",
		"KEYBINDS",
		" [UP ARROW] - Replaces input with previous command",
		" [DOWN ARROW] - Replaces input with the latter command",
		" [CTRL+C] - Quits typing when in command",
		" [CTRL+BACKSPACE] - Clears input",
		"",
		" More information is available in our documentation.",
	];

	//"SUBACCOUNTS"
	//"  subs - list all subaccounts"
	//"  sub [nickname] - change subaccount (case sensitive)"
	//ORDERS
	//"  ob | orderbook - get display of orderbook"
	//"  lev - get current account leverage"
	//"  lev [number] - change leverage to chosen number"

	for line in help_msg {
		us.prnt(line.to_string());
	}

	Ok(())
}

pub async fn about(us: &mut UserSpace) -> AnyHowResult<(), AnyHowError> {
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

	Ok(())
}

pub async fn config<B: Backend + std::marker::Send>(us: &mut UserSpace, terminal: &mut Terminal<B>) -> AnyHowResult<(), AnyHowError> {
	let mut database = Database::open(database_location().as_str()).unwrap();

	us.prnt(" 1. Change SLTP-Ratio warning number".to_string());
	let choice = us.ask_input("[Option Number]", terminal, Some("defaults_option_number")).await?;

	match choice.as_str() {
		"1" => {
			us.bl();
			let new_ratio =
				us.ask_input("[New ratio warning number]", terminal, None).await?.parse::<Decimal>()?;
			db_insert_config(&mut database, "ratio_warn_num", &new_ratio.to_string())?;
			us.prnt(" Changed ratio warning number successfully".to_string());
		}
		_ => {
			us.prnt("!! Not a choice !!".to_string());
		}
	}

	Ok(())
}

pub async fn sessions(us: &mut UserSpace) -> AnyHowResult<(), AnyHowError> {
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
	us.bl();

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

	Ok(())
}

pub async fn date(us: &mut UserSpace) -> AnyHowResult<(), AnyHowError> {
	us.prnt(format!("  {}", chrono::Utc::now().format("[UTC] %b %-d %Y")));
	us.prnt(format!("  {}", chrono::Local::now().format("[Now] %b %-d %Y")));

	Ok(())
}

pub async fn time(us: &mut UserSpace) -> AnyHowResult<(), AnyHowError> {
	us.prnt(format!("  {}", chrono::Utc::now().format("[UTC] %H:%M:%S")));
	us.prnt(format!("  {}", chrono::Local::now().format("[Now] %H:%M:%S")));

	Ok(())
}

pub async fn afetch(us: &mut UserSpace, command: &str) -> AnyHowResult<(), AnyHowError> {
	if command == "aclear" {
		us.clear_commands();
	}
	output_ascii(us);

	Ok(())
}

pub async fn ping_pong(us: &mut UserSpace) -> AnyHowResult<(), AnyHowError> {
	us.prnt(String::from("🏓 Pong!"));

	Ok(())
}

pub async fn command_count(us: &mut UserSpace) -> AnyHowResult<(), AnyHowError> {
	us.prnt(format!("{} Command{} this session", us.command_count, s_or_not(us.command_count as usize)));

	Ok(())
}

pub async fn cowsay(us: &mut UserSpace, command: &str) -> AnyHowResult<(), AnyHowError> {
	let message: String = command.split("cowsay ").collect();
	let mut top = String::new();
	let mut bottom = String::new();

	for _x in 1..message.len() {
		top += "_";
		bottom += "-";
	}

	let cow = [
		format!(" ___{}", top),
		format!("< {} >", message),
		format!(" ---{}", bottom),
		r"        \   ^__^".to_string(),
		r"         \  (oo)\".to_string(),
		r"            (__)\       )\/\".to_string(),
		r"                ||----w |".to_string(),
		r"                ||     ||".to_string()

	];

	for line in cow {
		us.prnt(line.to_string())
	}

	Ok(())
}

// Testing commands (for developer use)

pub async fn trade_fetch(us: &mut UserSpace) -> AnyHowResult<(), AnyHowError> {
	us.prnt(format!("{:?}", db_get_ftrades()?));

	Ok(())
}

pub async fn trade_wipe(us: &mut UserSpace) -> AnyHowResult<(), AnyHowError> {
	db_wipe_trades();
	us.prnt("WIPED TRADES".to_string());

	Ok(())
}