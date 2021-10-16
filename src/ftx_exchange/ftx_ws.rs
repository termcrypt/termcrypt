use ftx::options::Options;

use ftx::ws::Result;
use ftx::ws::{Channel, Data, Ws};
use futures::stream::StreamExt;

use super::super::utils::*;

pub async fn ftx_websocket(options: Options) -> Result<()> {
	let mut websocket = Ws::connect(options).await?;

	websocket
		.subscribe(vec![Channel::Fills, Channel::Orders])
		.await?;

	loop {
		let data = websocket.next().await.expect("No data received")?;

		match data {
			(_, Data::Fill(fill)) => {
				sideret(
					format!(
						"{:?} - Fill ({:?}) at: {} with size: {}",
						fill.market, fill.side, fill.price, fill.size
					)
					.as_str(),
				);
			}
			(_, Data::Order(_order)) => {
				//sideret(format!("{:?} - Order update at: {:?} with size: {} with status: {:?}", order.market, order.price, order.size, order.status).as_str());
			}
			_ => panic!("WebSocket: Unexpected data type"),
		}
	}
}
