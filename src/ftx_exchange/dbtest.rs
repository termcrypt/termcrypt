//this is a temporary file to test databases within termcrypt.
use anyhow::{
	Error,
	//bail
	Result,
};
//use chrono::{DateTime, Local};
use ftx::{options::Options, rest::*};
use polodb_bson::mk_document;
use polodb_core::Database;
use rand::Rng;
use rust_decimal::prelude::*;
//use rust_decimal_macros::dec;

use utils::{askout as ask, boldt};



let main_id = db_insert_ftrade(db::Trade {
    _id: None,
    timestamp_open: Local::now().timestamp().to_string().parse::<Decimal>()?,
    filled: false,
    risk,
    main_id: q_main_order.id,
    sl_id: None,
    tp_id: None,
})?;