use anyhow::{Error, Result};
/*
use futures::stream::StreamExt;
use polodb_bson::mk_document;
use polodb_core::Database;
use ftx::ws::Result;
use ftx::ws::{Channel, Data, Ws};
use rust_decimal::prelude::*;

use super::super::db::*;
use super::ftx_utils::*;
*/
use ftx::options::Options;

pub async fn init_websocket(_options: Options) -> Result<(), Error> {
	Ok(())
}
