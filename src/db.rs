use anyhow::{bail, Error, Result};
//use chrono::{DateTime, Local};
use ftx::{options::Options, rest::*};
use polodb_bson::mk_document;
use polodb_core::Database;
use rand::Rng;
use rust_decimal::prelude::*;
//use rust_decimal_macros::dec;

use super::utils::{askout as ask, boldt};

pub fn database_location() -> String {
	format!("{}/termcrypt/database", dirs::data_dir().unwrap().display())
}

pub fn history_location() -> String {
	format!("{}/termcrypt/history/", dirs::data_dir().unwrap().display())
}

pub async fn get_db_info(checkapi: bool) -> Result<super::Config, Error> {
	let mut db = Database::open(database_location().as_str()).unwrap();

	//set data point variables to specified db / default values
	let default_pair = get_dbinf_by_entry(&mut db, "default_pair", Some("BTC-PERP"), None, false)?;
	let default_sub = get_dbinf_by_entry(&mut db, "default_sub", Some("def"), None, false)?;
	let ratio_warn_num =
		get_dbinf_by_entry(&mut db, "ratio_warn_num", Some("1"), None, false)?.parse::<Decimal>()?;
	let mut ftx_pub_key;
	let mut ftx_priv_key;

	let mut force_retype = false;
	loop {
		ftx_pub_key = get_dbinf_by_entry(
			&mut db,
			"ftx_pub_key",
			None,
			Some("public FTX API key"),
			force_retype,
		)?;
		ftx_priv_key = get_dbinf_by_entry(
			&mut db,
			"ftx_priv_key",
			None,
			Some("private FTX API secret"),
			force_retype,
		)?;
		let api = Rest::new(Options {
			key: Some(ftx_pub_key.to_string()),
			secret: Some(ftx_priv_key.to_string()),
			subaccount: None,
			endpoint: ftx::options::Endpoint::Com,
		});

		if checkapi {
			match api.request(GetAccount {}).await {
				Ok(_x) => break,
				Err(e) => {
					println!();
					println!("{}", boldt(format!("{}", e).as_str()));
					println!(
						"  {}",
						boldt("!! API keys are not valid, please try again !!")
					);
					force_retype = true;
					continue;
				}
			}
		} else {
			break;
		}
	}
	Ok(super::Config {
		default_pair,
		default_sub,
		ratio_warn_num,
		ftx_pub_key,
		ftx_priv_key,
	})
}

pub fn get_dbinf_by_entry(
	db: &mut polodb_core::Database,
	key_name: &str,
	default_value: Option<&str>,
	name: Option<&str>,
	force_retype: bool,
) -> Result<String, Error> {
	//let mut db = Database::open(database_location().as_str()).unwrap();
	let mut collection = db.collection("config").unwrap();

	let value = if !force_retype {
		match collection
			.find_one(&mk_document! {"_key": key_name})
			.unwrap()
		{
			Some(val) => {
				//if value is found in collection
				val.get("value").unwrap().unwrap_string().to_string()
			}
			None => {
				//if value is not found in collection
				if let Some(default) = default_value {
					//if there is default and not required custom, return default
					default.to_string()
				} else {
					//if there is required value, ask user for input
					//print!("{}[2J", 27 as char);
					println!();
					println!(
						"{}",
						boldt("termcrypt needs configuration for first time use.")
					);
					println!();
					let input = ask(&format!("Please enter your {}", name.unwrap()), None)?;
					db_insert_config(db, key_name, &input)?
				}
			}
		}
	} else {
		//this is to retype if required value was not valid
		let input = ask(&format!("Please enter your {}", name.unwrap()), None)?;
		db_insert_config(db, key_name, &input)?
	};
	Ok(value)
}

pub fn db_insert_config(db: &mut polodb_core::Database, key_name: &str, value: &str) -> Result<String, Error> {
	let mut document = mk_document! {
		"_key": key_name,
		"value": value
	};

	let inside = db_inside(db, document.as_mut());

	let mut collection = db.collection("config").unwrap();
	//checks if key already has a value
	if inside {
		//updates the database entry with new values
		collection
			.update(
				Some(&mk_document! { "_key": key_name }),
				&mk_document! {
				   "$set": mk_document! {
					  "value": value
				   }
				},
			)
			.unwrap();
	} else {
		//inserts a new database entry
		collection.insert(&mut document).unwrap();
	}

	Ok(value.to_string())
}

pub fn db_inside(db: &mut polodb_core::Database, bruh: &mut polodb_bson::Document) -> bool {
	let mut collection = db.collection("config").unwrap();

	match collection.find_one(bruh).unwrap() {
		Some(_val) => true,
		None => false,
	}
}

#[derive(Debug)]
pub enum Exchange {
	Ftx,
	Bybit,
}

#[derive(Debug)]
pub struct Trade {
	pub _id: Option<Decimal>,
	pub exchange_id: Option<Decimal>,
	pub timestamp_open: Decimal,
	pub filled: bool,
	pub risk: Decimal,
	pub main_id: u64,
	pub sl_id: Option<u64>,
	pub tp_id: Option<u64>,
	pub exchange: Exchange,
}

pub fn db_insert_ftrade(td: Trade) -> Result<i64, Error> {
	let mut db = Database::open(database_location().as_str()).unwrap();
	let mut collection = db.collection("ftrades").unwrap();

	let all_trades = collection.find_all().unwrap();
	let mut id: i64;
	//makes sure id is not already in ftrades
	loop {
		id = rand::thread_rng().gen_range(10..999999);
		let mut failed = false;
		for trade in &all_trades {
			if id == trade.get("_id").unwrap().unwrap_int() {
				failed = true
			}
		}
		if !failed {
			break;
		}
	}

	collection
		.insert(&mut mk_document! {
			"_id": id,
			"exchange_id": if td.exchange_id == None {"".to_string()} else {td.exchange_id.unwrap().to_string()},
			"timestamp_open": td.timestamp_open.to_string(),
			"filled": if td.filled {"true"} else {"false"},
			"risk": td.risk.to_string(),
			"main_id": td.main_id,
			"sl_id": if td.sl_id == None {0} else {td.sl_id.unwrap()},
			"tp_id": if td.tp_id == None {0} else {td.tp_id.unwrap()},
			"exchange": match td.exchange {
				Exchange::Ftx => {"ftx"},
				Exchange::Bybit => {"bybit"},
			}
		})
		.unwrap();
	Ok(id)
}

pub fn _db_get_ftrade(id: i64) -> Result<Option<Trade>, Error> {
	let mut db = Database::open(database_location().as_str()).unwrap();
	let mut collection = db.collection("ftrades").unwrap();
	match collection.find_one(&mk_document! {"_id": id}).unwrap() {
		Some(doc) => {
			//if value is found in collection
			let sl_id = doc.get("sl_id").unwrap().unwrap_int();
			let tp_id = doc.get("tp_id").unwrap().unwrap_int();
			let exchange = doc.get("exchange").unwrap().unwrap_string();
			let exchange_id = doc.get("exchange_id").unwrap().unwrap_string();
			let filled = doc.get("filled").unwrap().unwrap_string();

			Ok(Some(Trade {
				_id: Some(doc.get("_id").unwrap().unwrap_int().to_string().parse::<Decimal>()?),
				exchange_id: if exchange_id == "" { None } else {Some(exchange_id.parse::<Decimal>()?)},
				timestamp_open: /*DateTime::parse_from_str(*/doc.get("timestamp_open").unwrap().unwrap_string().parse::<Decimal>()?/*, "%s")?*/,
				filled: if filled == "true" {true} else {false},
				risk: doc.get("risk").unwrap().unwrap_string().parse::<Decimal>()?,
				main_id: doc.get("main_id").unwrap().unwrap_int() as u64,
				sl_id: if sl_id == 0 {None} else {Some(sl_id as u64)},
				tp_id: if tp_id == 0 {None} else {Some(tp_id as u64)},
				exchange: match exchange {
					"ftx" => {Exchange::Ftx},
					"bybit" => {Exchange::Bybit},
					_ => {bail!(format!("Exchange option: <{}> is not a valid exchange.", exchange))}
				}
			}))
		}
		None => {
			//if value is not found in collection
			Ok(None)
		}
	}
}

pub fn db_wipe_trades() {
	let mut db = Database::open(database_location().as_str()).unwrap();
	db.collection("ftrades").unwrap().delete(None).unwrap();
}

pub fn db_get_ftrades() -> Result<Vec<Trade>, Error> {
	let mut db = Database::open(database_location().as_str()).unwrap();
	let mut collection = db.collection("ftrades").unwrap();
	let all_trades = collection.find_all().unwrap();
	let mut trade_array: Vec<Trade> = Vec::with_capacity(all_trades.len());

	for doc in all_trades {
		let sl_id = doc.get("sl_id").unwrap().unwrap_int();
		let tp_id = doc.get("tp_id").unwrap().unwrap_int();
		let exchange = doc.get("exchange").unwrap().unwrap_string();
		let exchange_id = doc.get("exchange_id").unwrap().unwrap_string();
		let filled = doc.get("filled").unwrap().unwrap_string();

		trade_array.push(
			Trade {
				_id: Some(doc.get("_id").unwrap().unwrap_int().to_string().parse::<Decimal>()?),
				exchange_id: if exchange_id == "" { None } else {Some(exchange_id.parse::<Decimal>()?)},
				timestamp_open: /*DateTime::parse_from_str(*/doc.get("timestamp_open").unwrap().unwrap_string().parse::<Decimal>()?/*, "%s")?*/,
				filled: if filled == "true" {true} else {false},
				risk: doc.get("risk").unwrap().unwrap_string().parse::<Decimal>()?,
				main_id: doc.get("main_id").unwrap().unwrap_int() as u64,
				sl_id: if sl_id == 0 {None} else {Some(sl_id as u64)},
				tp_id: if tp_id == 0 {None} else {Some(tp_id as u64)},
				exchange: match doc.get("exchange").unwrap().unwrap_string() {
					"ftx" => {Exchange::Ftx},
					"bybit" => {Exchange::Bybit},
					_ => {bail!(format!("Exchange option: <{}> is not a valid exchange.", exchange))}
				}
			}
		);
	}

	Ok(trade_array)
}
