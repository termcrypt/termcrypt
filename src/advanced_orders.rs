use anyhow::{
    Result,
    Error,
    bail
};

use ftx::{
    rest::Rest,
    //options::Options,
    //rest::Account
};

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
    pub orderbookpos: Option<Decimal>
}

async fn makeorder(o:&NowOrder, api:&mut Rest) -> Result<ftx::rest::OrderInfo, Error> {
    let bruh = api.place_order(
        &o.pair,
        if o.islong {ftx::rest::Side::Buy} else {ftx::rest::Side::Sell},
        o.entry,
        ftx::rest::OrderType::Limit, 
        o.real_quantity,
        None, //
        None,
        None, //
        None
    ).await?;
    Ok(bruh)
}

//for orders that
pub async fn o_now_order(mut o:NowOrder, api:&mut Rest) -> Result<ftx::rest::OrderInfo, Error> {
    Ok(
        match o.entry{
            Some(entry) if entry > o.price => {
                api.place_trigger_order(
                    &o.pair,
                    if o.islong {ftx::rest::Side::Buy} else {ftx::rest::Side::Sell},
                    o.real_quantity,
                    ftx::rest::OrderType::Stop,
                    entry,
                    Some(true),
                    None,
                    None,
                    None
                ).await?
            },
            _ => {
                if o.ismarket {
                    api.place_order(
                        &o.pair,
                        if o.islong {ftx::rest::Side::Buy} else {ftx::rest::Side::Sell},
                        None,
                        ftx::rest::OrderType::Market,
                        o.real_quantity,
                        None,
                        None,
                        None,
                        None
                    ).await?
                } else if o.isorderbook {
                    //start aggressive limit order
                    //let mut _i:i32 = 0;
                    for mut _i in 1..10 {
                        let q_orderbook = api.get_orderbook(&o.pair.as_str(), Some(10)).await?;
                        if o.islong {
                            o.entry = Some(q_orderbook.bids[o.orderbookpos.unwrap().to_usize().unwrap()].0);
                        } else {
                            o.entry = Some(q_orderbook.asks[o.orderbookpos.unwrap().to_usize().unwrap()].0);
                        }

                        let order = makeorder(&o, api).await;
                        _i += 1;
                        if order.is_ok() {
                            return order;
                            //break
                        }
                        /*
                        if order.is_err() {
                            println!("{:#?}", order)
                        }
                        */
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
        }
    )
}

//for orders that don't need fast retries (future setup)
pub fn _o_tobe_order() {

}

//makes sure either you get in or get out the market with a certain slippage
pub fn _o_aggressive_order() {

}