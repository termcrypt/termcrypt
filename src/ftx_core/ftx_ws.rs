use anyhow::{Error as AnyHowError, Result as AnyHowResult};
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

pub async fn ftx_websocket(options: Options) -> AnyHowResult<(), AnyHowError> {
	Ok(())
}
