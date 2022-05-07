use crate::db::*;
use anyhow::{Error, Result, Context};
use chrono::{
	DateTime,
	Duration,
	NaiveTime,
	TimeZone,
	Utc
};
use unicode_width::UnicodeWidthStr;
use polodb_core::Database;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use tui::{
	backend::Backend,
    Terminal,
};

use crate::{
	Exchange,
	UserSpace,
	utils::{
		round_dp_tz,
		output_ascii,
		s_or_not
	}
};

pub async fn help(us: &mut UserSpace) -> Result<(), Error> {
	// Dynamically listing these would be "harder" somebody said. Oh well...
	let help_msg = [
		"UTILITY",
		"  help -> Get information about commands",
		"  about / info -> Information about the project",
		"  afetch / aclear -> Ascii art with info",
		"  clear / clr -> Clear the terminal",
		"  date -> Get current local and UTC date",
		"  time -> Get current local and UTC time",
		"  sessions / ses -> Get trading session times",
		"",
		"MARKETS",
		"  balance / bal -> Get balances of subaccount",
		"  search / search [query] -> Query pairs",
		"  pair / pair [query] -> Lets you change the current pair",
		"  price / p -> Return mark, ask and bid price for pair",
		"",
		"ORDERS",
		"  order / o -> Start an order",
		"  m -> Start a market order",
		"  l -> Start a limit order",
		"  ob -> Start an OB-based order",
		"",
		"SETTINGS",
		"  defaults / def -> Change startup defaults (exchange specific)",
		"  config / conf -> Change configuration settings",
		"",
		"KEYBINDS",
		"  [UP ARROW] -> Replaces input with previous command",
		"  [DOWN ARROW] -> Replaces input with the latter command",
		"  [CTRL+C] -> Quits typing when in command or quits app",
		"  [CTRL+BACKSPACE] -> Clears input",
		"",
		"More information is available in our documentation.",
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

pub async fn about(us: &mut UserSpace) -> Result<(), Error> {
	let about = [
		String::new(),
		"About termcrypt".to_string(),
		"  termcrypt is a project created for managing efficient trading.".to_string(),
		"  It is a free, open sourced app licensed under AGPL3+ (a copyleft license).".to_string(),
		"  You can visit our project repository at https://github.com/termcrypt/termcrypt".to_string(),
		String::new(),
		format!(" You are running termcrypt version: {}", super::VERSION)
	];

	for line in about {
		us.prnt(line);
	}

	Ok(())
}

pub async fn config<B: Backend + std::marker::Send>(us: &mut UserSpace, terminal: &mut Terminal<B>) -> Result<(), Error> {
	let mut database = Database::open(database_location().as_str()).context("")?;

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

pub async fn sessions(us: &mut UserSpace) -> Result<(), Error> {
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

pub async fn date(us: &mut UserSpace) -> Result<(), Error> {
	us.prnt(format!("  {}", chrono::Utc::now().format("UTC: %b %-d %Y")));
	us.prnt(format!("  {}", chrono::Local::now().format("Now: %b %-d %Y")));

	Ok(())
}

pub async fn time(us: &mut UserSpace) -> Result<(), Error> {
	us.prnt(format!("  {}", chrono::Utc::now().format("UTC: %H:%M:%S")));
	us.prnt(format!("  {}", chrono::Local::now().format("Now: %H:%M:%S")));

	Ok(())
}

pub async fn afetch(us: &mut UserSpace, command: &str) -> Result<(), Error> {
	if command == "aclear" {
		us.clear_commands();
	}
	output_ascii(us);

	Ok(())
}

pub async fn ping_pong(us: &mut UserSpace) -> Result<(), Error> {
	us.prnt(String::from("🏓 Pong!"));

	Ok(())
}

pub async fn command_count(us: &mut UserSpace) -> Result<(), Error> {
	us.prnt(format!("{} Command{} this session", us.command_count, s_or_not(us.command_count as usize)));

	Ok(())
}

pub async fn cowsay(us: &mut UserSpace, command: &str) -> Result<(), Error> {
	let message: String = command.split("cowsay ").collect();
	let top = "_".repeat(message.width());
	let bottom = "-".repeat(message.width());

	let cow = [
		format!(" __{}", top),
		format!("< {} >", message),
		format!(" --{}", bottom),
		r"        \   ^__^".to_string(),
		r"         \  (oo)\_______".to_string(),
		r"            (__)\       )\/\".to_string(),
		"                ||----w |".to_string(),
		"                ||     ||".to_string()
	];

	for line in cow {
		us.prnt(line.to_string())
	}

	Ok(())
}

// Testing commands (for developer use)

pub async fn trade_fetch(us: &mut UserSpace) -> Result<(), Error> {
	us.prnt(format!("{:?}", db_get_ftrades()?));

	Ok(())
}

pub async fn trade_wipe(us: &mut UserSpace) -> Result<(), Error> {
	db_wipe_trades()?;
	us.prnt("WIPED TRADES".to_string());

	Ok(())
}

pub async fn switch_exchange(us: &mut UserSpace, command: &str) -> Result<(), Error> {
	let new_exchange: String = command.to_lowercase().split("switch ").collect();
	let mut new_exchange_type: Option<Exchange> = None;

	for enum_item in Exchange::VALUES {
		if enum_item.to_string().to_lowercase() == new_exchange {
			new_exchange_type = Some(enum_item);
		}
	}

	if new_exchange_type.is_none() {
		us.prnt(format!(" {} is not an exchange!", new_exchange));
	} else {
		us.switch_exchange(new_exchange_type.context("")?).await?;
		us.prnt(format!(" Switched to {} exchange", us.active_exchange));
	}

	Ok(())
}