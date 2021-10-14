use anyhow::{bail, Error, Result};

use ftx::rest::*;

use rust_decimal::prelude::*;
/*
use rust_decimal_macros::{
	dec
};*/

pub struct NowOrder {
	pub pair: String,
	pub islong: bool,
	pub real_quantity: Decimal,
	pub ismarket: bool,
	pub entry: Option<Decimal>,
	pub price: Decimal,
	pub isorderbook: bool,
	pub orderbookpos: Option<Decimal>,
}

pub struct SLOrder {
	pub pair: String,
	pub islong: bool,
	pub real_quantity: Decimal,
	pub stop_price: Decimal,
	pub sl_type: SLType,
}

pub struct TPOrder {
	pub pair: String,
	pub islong: bool,
	pub real_quantity: Decimal,
	pub tp_price: Decimal,
	pub tp_type: TPType,
}

pub enum SLType {
	M,  //market
	Hs, //hardsoft,
}

pub enum TPType {
	M,  //market
	Ob, //orderbook limit
}

async fn makeorder(o: &NowOrder, api: &mut Rest) -> Result<ftx::rest::OrderInfo, Error> {
	let bruh = api
		.request(PlaceOrder {
			market: o.pair.to_string(),
			side: if o.islong {
				ftx::rest::Side::Buy
			} else {
				ftx::rest::Side::Sell
			},
			price: o.entry,
			r#type: ftx::rest::OrderType::Limit,
			size: o.real_quantity,
			client_id: None,
			ioc: false,
			post_only: true,
			reduce_only: false,
			reject_on_price_band: false,
		})
		.await?;
	Ok(bruh)
}

//for orders that
pub async fn o_now_order(mut o: NowOrder, api: &mut Rest) -> Result<ftx::rest::OrderInfo, Error> {
	Ok(match o.entry {
		//for limit entries on opposite side of market
		Some(entry)
			if (entry > o.price && o.islong)
				|| (entry < o.price && !o.islong) && !o.isorderbook =>
		{
			api.request(PlaceTriggerOrder {
				market: o.pair.to_string(),
				side: if o.islong {
					ftx::rest::Side::Buy
				} else {
					ftx::rest::Side::Sell
				},
				size: o.real_quantity,
				r#type: ftx::rest::OrderType::Stop,
				trigger_price: entry,
				reduce_only: None,
				order_price: None,
				retry_until_filled: None,
				trail_value: None,
			})
			.await?
		}
		//for normal limit entries
		Some(entry)
			if (entry < o.price && o.islong)
				|| (entry > o.price && !o.islong) && !o.isorderbook =>
		{
			makeorder(&o, api).await?
		}
		_ => {
			//for immediate market orders
			if o.ismarket {
				api.request(PlaceOrder {
					market: o.pair.to_string(),
					side: if o.islong {
						ftx::rest::Side::Buy
					} else {
						ftx::rest::Side::Sell
					},
					price: None,
					r#type: ftx::rest::OrderType::Market,
					size: o.real_quantity,
					client_id: None,
					ioc: false,
					post_only: false,
					reduce_only: false,
					reject_on_price_band: false,
				})
				.await?
			//for orderbook based immediate limit order
			} else if o.isorderbook {
				for mut _i in 1..10 {
					let q_orderbook = api
						.request(GetOrderBook {
							market_name: o.pair.to_string(),
							depth: Some(10),
						})
						.await?;
					if o.islong {
						o.entry =
							Some(q_orderbook.bids[o.orderbookpos.unwrap().to_usize().unwrap()].0);
					} else {
						o.entry =
							Some(q_orderbook.asks[o.orderbookpos.unwrap().to_usize().unwrap()].0);
					}

					let order = makeorder(&o, api).await;
					_i += 1;
					if order.is_ok() {
						return order;
					}
					/*if order.is_err() {
						println!("{:#?}", order)
					}*/
					if _i == 10 {
						println!("{:#?}", order);
					} else {
						println!("Trying order again");
					}
				}
				bail!("Order failed after multiple tries.")
			} else {
				bail!("No order types supported for function yet")
			}
		}
	})
}

//use dise later ;))
pub fn _o_aggressive_order() {}

pub async fn o_sl_order(o: SLOrder, api: &mut Rest) -> Result<ftx::rest::OrderInfo, Error> {
	Ok(match o.sl_type {
		SLType::M => {
			api.request(PlaceTriggerOrder {
				market: o.pair.to_string(),
				side: if o.islong {
					ftx::rest::Side::Sell
				} else {
					ftx::rest::Side::Buy
				},
				size: o.real_quantity,
				r#type: ftx::rest::OrderType::Stop,
				trigger_price: o.stop_price,
				reduce_only: Some(true),
				order_price: None,
				retry_until_filled: None,
				trail_value: None,
			})
			.await?
		}
		SLType::Hs => bail!("Type not ready yet."),
	})
}

pub async fn o_tp_order(o: TPOrder, api: &mut Rest) -> Result<ftx::rest::OrderInfo, Error> {
	Ok(match o.tp_type {
		TPType::M => {
			api.request(PlaceTriggerOrder {
				market: o.pair.to_string(),
				side: if o.islong {
					ftx::rest::Side::Sell
				} else {
					ftx::rest::Side::Buy
				},
				size: o.real_quantity,
				r#type: ftx::rest::OrderType::TakeProfit,
				trigger_price: o.tp_price,
				reduce_only: Some(true),
				order_price: None,
				retry_until_filled: None,
				trail_value: None,
			})
			.await?
		}
		TPType::Ob => {
			bail!("Type not ready yet.")
		}
	})
}
