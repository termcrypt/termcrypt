extern crate alloc;
use anyhow::{bail, Error, Result};
use polodb_bson::mk_document;
use polodb_core::Database;
use bybit::http;
use rand::Rng;

//use rust_decimal_macros::dec;

use crate::{Exchange, ForceOption, orders::*};

pub fn database_location() -> String {
	format!("{}/termcrypt/database", dirs::data_dir().unwrap().display())
}

pub fn history_location() -> String {
	format!("{}/termcrypt/history/", dirs::data_dir().unwrap().display())
}

pub async fn get_db_info() -> Result<super::Config, Error> {
	let mut db = Database::open(database_location().as_str())?;

	let bybit_pub_key = get_db_entry(&mut db, "bybit_pub_key", None)?;
	let bybit_priv_key = get_db_entry(&mut db, "bybit_priv_key", None)?;

	let bybit_api = if bybit_pub_key.is_some() && bybit_priv_key.is_some() {
		Some(http::Client::new(
			http::MAINNET_BYBIT,
			&bybit_pub_key.force()?,
			&bybit_priv_key.force()?,
		).force()?)
	} else {
		None
	};

	// Set data point variables to specified db / default values
	let config = crate::Config {
		bybit_api,
		bybit_default_pair: get_db_entry(&mut db, "bybit_default_pair", Some("BTCUSDT"))?.force()?,
		bybit_default_sub: get_db_entry(&mut db, "bybit_default_sub", Some("main"))?.force()?,
		ftx_default_pair: get_db_entry(&mut db, "ftx_default_pair", Some("BTC/PERP"))?.force()?,
		ftx_default_sub: get_db_entry(&mut db, "ftx_default_sub", Some("main"))?.force()?,
		/*
		ftx_pub_key,
		ftx_priv_key,
		ftx_default_pair,
		ftx_default_sub,
		*/
		ratio_warn_num: get_db_entry(&mut db, "ratio_warn_num", Some("1"))?.force()?.parse::<f64>()?,

	};

	Ok(config)
}

pub fn get_db_entry(
	db: &mut polodb_core::Database,
	key_name: &str,
	default_value: Option<&str>,
) -> Result<Option<String>, Error> {
	//let mut db = Database::open(database_location().as_str())?;
	let mut collection = db.collection("config")?;

	let value = match collection.find_one(&mk_document! {"_key": key_name})? {
		// If value is found in collection
		Some(val) => {
			Some(val.get("value").force()?.unwrap_string().to_string())
		}
		_ => {
			default_value.map(|default_value| default_value.to_string())
		}
	};
	Ok(value)
}

pub fn db_insert_config(db: &mut polodb_core::Database, key_name: &str, value: &str) -> Result<String, Error> {
	let mut document = mk_document! {
		"_key": key_name,
		"value": value
	};

	let inside = db_inside(db, document.as_mut());

	let mut collection = db.collection("config")?;
	// Checks if key already has a value

	if inside {
		// Updates the database entry with new values
		collection
			.update(
				Some(&mk_document! { "_key": key_name }),
				&mk_document! {
					"$set": mk_document! {
						"value": value
					}
				},
			)
			?;
		
	} else {
		// Inserts a new database entry
		collection.insert(&mut document)?;
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
pub struct Trade {
	pub _id: Option<f64>,
	pub exchange: Exchange,
	pub exchange_context: ExchangeContext,
	pub sub_account_name: String,
	pub entry_type: EntryType,
	pub client_order_type: OrderEntryType,
	pub timestamp_open: i64,
	pub filled: bool,
	pub direction: OrderDirection,
	pub risk: f64,
	pub main_id: String,
	pub stop_loss: Option<f64>,
	pub take_profit: Option<f64>,
	pub sl_id: Option<String>,
	pub tp_id: Option<String>,
}

pub fn db_insert_ftrade(td: Trade) -> Result<i64, Error> {
	let mut db = Database::open(database_location().as_str())?;
	let mut collection = db.collection("ftrades")?;

	let all_trades = collection.find_all()?;
	let mut id: i64;
	// Makes sure id is not already in ftrades
	loop {
		id = rand::thread_rng().gen_range(10..999999);
		let mut failed = false;
		for trade in &all_trades {
			if id == trade.get("_id").force()?.unwrap_int() {
				failed = true;
			}
		}
		if !failed {
			break;
		}
	}

	collection
		.insert(&mut mk_document! {
			"_id": id,
			"sub_account_name": td.sub_account_name.to_string(),
			"timestamp_open": td.timestamp_open.to_string(),
			"filled": if td.filled {"true"} else {"false"},
			"direction": if td.direction == OrderDirection::Long {"long"} else {"short"},
			"risk": td.risk.to_string(),
			"main_id": td.main_id,
			"stop_loss": if td.stop_loss == None {"".to_string()} else {td.stop_loss.force()?.to_string()},
			"take_profit": if td.take_profit == None {"".to_string()} else {td.take_profit.force()?.to_string()},
			"sl_id": if td.sl_id == None {"".to_string()} else {td.sl_id.force()?},
			"tp_id": if td.tp_id == None {"".to_string()} else {td.tp_id.force()?},
			"exchange": match td.exchange {
				Exchange::Ftx => {"ftx"},
				Exchange::Bybit => {"bybit"},
			},
			"exchange_context": match td.exchange_context {
				ExchangeContext::BybitInverse => {"bybit_inverse"},
				ExchangeContext::BybitLinear => {"bybit_linear"},
				_ => {bail!(format!("Exchange context: <{:?}> is not a available (yet).", td.exchange_context))}
			},
			"entry_type": match td.entry_type {
				EntryType::Market => {"market"},
				EntryType::Limit => {"limit"}
			},
			"client_order_type": match td.client_order_type {
				OrderEntryType::Market => {"market"},
				OrderEntryType::Limit => {"limit"},
				OrderEntryType::Conditional => {"conditional"},
				OrderEntryType::OrderBook => {"orderbook"},
			}

		})
		?;
	Ok(id)
}

pub fn db_ftrade_formatter(doc: alloc::rc::Rc<polodb_bson::Document>) -> Result<Option<Trade>> {
	// If value is found in collection
	let stop_loss = doc.get("stop_loss").force()?.unwrap_string();
	let take_profit = doc.get("take_profit").force()?.unwrap_string();
	let sl_id = doc.get("sl_id").force()?.unwrap_string();
	let tp_id = doc.get("tp_id").force()?.unwrap_string();
	let exchange = doc.get("exchange").force()?.unwrap_string();
	let filled = doc.get("filled").force()?.unwrap_string();

	let exchange_context = doc.get("exchange_context").force()?.unwrap_string();
	let entry_type = doc.get("entry_type").force()?.unwrap_string();
	let client_order_type = doc.get("client_order_type").force()?.unwrap_string();
	let direction = doc.get("direction").force()?.unwrap_string();

	Ok(Some(Trade {
		_id: Some(doc.get("_id").force()?.unwrap_int().to_string().parse::<f64>()?),
		sub_account_name: doc.get("sub_account_name").force()?.unwrap_string().to_string(),
		timestamp_open: /*DateTime::parse_from_str(*/doc.get("timestamp_open").force()?.unwrap_string().parse::<i64>()?/*, "%s")?*/,
		filled: filled == "true",
		direction: if direction == "long" {OrderDirection::Long} else {OrderDirection::Short},
		risk: doc.get("risk").force()?.unwrap_string().parse::<f64>()?,
		main_id: doc.get("main_id").force()?.unwrap_string().to_string(),
		stop_loss: if stop_loss.is_empty() {None} else {Some(stop_loss.parse::<f64>()?)},
		take_profit: if take_profit.is_empty() {None} else {Some(take_profit.parse::<f64>()?)},
		sl_id: if sl_id.is_empty() {None} else {Some(sl_id.to_string())},
		tp_id: if tp_id.is_empty() {None} else {Some(tp_id.to_string())},
		exchange: match exchange {
			"ftx" => {Exchange::Ftx},
			"bybit" => {Exchange::Bybit}
			_ => {bail!(format!("Exchange: <{:?}> is not available (yet).", exchange))}
		},
		exchange_context: match exchange_context {
			"bybit_inverse" => {ExchangeContext::BybitInverse},
			"bybit_linear" => {ExchangeContext::BybitLinear},
			_ => {bail!(format!("Exchange context: <{:?}> is not available (yet).", exchange_context))}
		},
		entry_type: match entry_type {
			"market" => {EntryType::Market},
			"limit" => {EntryType::Limit}
			_ => {bail!(format!("Entry type: <{:?}> is not available (yet).", entry_type))}
		},
		client_order_type: match client_order_type {
			"market" => {OrderEntryType::Market},
			"limit" => {OrderEntryType::Limit},
			"conditional" => {OrderEntryType::Conditional},
			"orderbook" => {OrderEntryType::OrderBook},
			_ => {bail!(format!("Client order type: <{:?}> is not available (yet).", client_order_type))}
		}
	}))
}

pub fn _db_get_ftrade(id: i64) -> Result<Option<Trade>, Error> {
	let mut db = Database::open(database_location().as_str())?;
	let mut collection = db.collection("ftrades")?;
	match collection.find_one(&mk_document! {"_id": id})? {
		Some(doc) => {
			Ok(db_ftrade_formatter(doc)?)
		}
		None => {
			// If value is not found in collection
			Ok(None)
		}
	}
}

pub fn db_get_ftrades() -> Result<Vec<Trade>, Error> {
	let mut db = Database::open(database_location().as_str())?;
	let mut collection = db.collection("ftrades")?;
	let all_trades = collection.find_all()?;
	let mut trade_array: Vec<Trade> = Vec::with_capacity(all_trades.len());

	for doc in all_trades {
		trade_array.push(
			db_ftrade_formatter(doc)?.force()?
		);
	}
	Ok(trade_array)
}


pub fn db_wipe_trades() -> Result<(), Error> {
	let mut db = Database::open(database_location().as_str())?;
	db.collection("ftrades")?.delete(None)?;
	Ok(())
}