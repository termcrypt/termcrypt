use ftx::options::Options;
use ftx::ws::Result;
use ftx::ws::{Channel, Data, Ws};
use futures::stream::StreamExt;
use polodb_bson::mk_document;
use polodb_core::Database;
use rust_decimal::prelude::*;

use super::super::db::*;
use super::super::utils::*;
use super::ftx_utils::*;

pub async fn ftx_websocket(options: Options) -> Result<()> {
	let mut websocket = Ws::connect(options).await?;

	websocket
		.subscribe(vec![Channel::Fills, Channel::Orders])
		.await?;

	loop {
		let data = websocket.next().await.expect("No data received")?;

		match data {
			(_, Data::Fill(fill)) => {
				//prints fill to user
				sideret(
					format!(
						"{:?} - Fill ({:?}) at: {} with size: {} {}",
						fill.market, fill.side, fill.price, fill.price*fill.size, ftx_getsuffixsymbol(&fill.market)
					)
					.as_str(),
				);
				//println!("{:#?}", fill);

				//updates entry to fill if any
				for item in db_get_ftrades().unwrap() {
					if item.exchange_id != None
						&& item.exchange_id.unwrap()
							== fill.id.to_string().parse::<Decimal>().unwrap()
					{
						let mut db = Database::open(database_location().as_str()).unwrap();
						let mut collection = db.collection("ftrades").unwrap();
						let id = item._id.unwrap().to_string();

						collection
							.update(
								Some(&mk_document! { "_id": id }),
								&mk_document! {
									"$set": mk_document! {
										"filled": "true"
									}
								},
							)
							.unwrap();
					}
				}
			}
			(_, Data::Order(_order)) => {
				//sideret(format!("{:?} - Order update at: {:?} with size: {} with status: {:?}", order.market, order.price, order.size, order.status).as_str());
			}
			_ => panic!("WebSocket: Unexpected data type"),
		}
	}
}
