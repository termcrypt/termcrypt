use anyhow::{bail, Error as AnyHowError, Result};
use bybit::{http, rest::*, Result as BybitResult, OrderType, Side, TimeInForce, Symbol};
use rust_decimal::prelude::*;
use chrono::{Utc};
use rust_decimal_macros::dec;
use bybit::Error;
use polodb_core::Database;

use super::super::misc;
use super::super::utils::{askout as ask, boldt, round_dp_tz, yn, bl};
use super::bybit_utils::*;
use super::bybit_advanced_orders::*;
use super::super::db;
use super::super::db::*;

pub struct CommandHandling<'a> {
	pub command_input: &'a str,
	pub current_sub_account: &'a mut String,
	pub current_pair: &'a mut String,
	pub bybit_api: &'a mut http::Client,
	pub _terminal_is_wide: &'a mut bool,
	pub database_info: &'a mut super::super::Config,
}

//Command Handling
pub async fn handle_commands<'a>(ch: CommandHandling<'_>) -> Result<bool, AnyHowError> {
	//defining variables from struct
	let x = ch.command_input;
	let sub_account = ch.current_sub_account;
	let pair = ch.current_pair;
	let api = ch.bybit_api;
	let _is_wide = ch._terminal_is_wide;
	let db_info = ch.database_info;

	let mut is_real_command = true;
	//handles the command given by the user
	match x {
		//lists all commands
		"p" | "price" => {
			let q_tickers = api.fetch_tickers(pair).await?;
			let ticker = q_tickers.tickers().next().unwrap();
			println!("  Mid: {}", ticker.last_price);
			println!("  Ask: {}", ticker.ask_price);
			println!("  Bid: {}", ticker.bid_price);
		},
		//change the current pair
		x if x.starts_with("pair") => { 
			let mut joined_pair: String;

			if x.starts_with("pair ") {
				joined_pair = x.split("pair ").collect();
			} else {
				//accept user input in two parts
				println!("  Change pair:");
				let prefix = ask("[Prefix]", Some("prefixpair".to_string()))?;
				let suffix = ask("[Suffix]", Some("suffixpair".to_string()))?;
			
				joined_pair = format!("{}{}", prefix, suffix);
			}

			joined_pair = joined_pair.to_uppercase();

			let q_symbols = api.fetch_symbols().await?;
			let mut is_real_pair: bool = false;

			for symbol in q_symbols.symbols() {
				if symbol.alias == joined_pair {
					is_real_pair = true;
				}
			}

			match is_real_pair {
				true => {
					println!("    {}", boldt("Switched (pair found)"));
					let q_tickers = api.fetch_tickers(&joined_pair).await?;
					let ticker = q_tickers.tickers().next().unwrap();

					//changes global pair value to new chosen pair
					*pair = joined_pair;
					println!("    Price ({}): {}", pair, ticker.last_price);

					//debug
					println!("{:#?}", ticker);
				}
				false => {
					println!("    {}", boldt("Error (pair not found)"));
				}
			}
		},
		//search for market by query
		x if x.starts_with("search") => {
			let to_search: String;

			if x.starts_with("search "){
				to_search = x.split("search ").collect();
			} else {
				to_search = ask("[Query Pairs]", Some("searchpair".to_string()))?;
			}
			//grabs second part of command to search for
			let q_symbols = api.fetch_symbols().await?;
			let symbols = q_symbols.symbols();
			bl();

			let mut matched_count: i32 = 0;
			//loop over all markets
			for symbol in symbols {
				if symbol.alias.contains(&to_search.to_uppercase()) {
					//presents to user the match found
					println!("HIT: {}", symbol.alias);
					//increases matched search counter
					matched_count += 1;
				}
			}
			//gives result of search operation
			println!("  {}", boldt(format!("{} Pairs Found", matched_count).as_str()));
		},
		"m"|"l"|"o" => {
			/*
			//  Checking ordertype
			*/

			let order_type: OrderEntryType;
			match x {
				"m" => {
					order_type = OrderEntryType::Market;
				},
				"l" => {
					order_type = OrderEntryType::Limit;
				},
				_ => {
					let mut temp_order_type: String;
					loop {
						temp_order_type = ask("[m|l]", Some("orderentrytype".to_string()))?;
						match temp_order_type.as_str() {
							"m" => {
								order_type = OrderEntryType::Market;
							},
							"l" => {
								order_type = OrderEntryType::Limit;
							},
							_ => {
								bl();
								println!("  !!! NOT AN OPTION !!! Please choose market or limit (m or l)");
								bl();
								continue;
							}
						}
						break;
					}
				}
			}

			let base_currency = bybit_get_base(pair.to_owned())?;
			let alt_base: String;
			let mut long_dec_needed = true;
			let mut is_linear = true;

			if pair.starts_with(&base_currency) {
				alt_base = pair.strip_prefix(&base_currency).unwrap().to_string();
				is_linear = false;
			} else {
				alt_base = pair.strip_suffix(&base_currency).unwrap().to_string();
				long_dec_needed = false;
			}

			let mut total_liquid: f64 = 0.0;
			let mut available_liquid: f64 = 0.0;
			let mut found_currency = false;

			let wallets = api.fetch_wallets().await?;

			//This infers that risk is based on singular balance of coin. It may not spread across inverse perpetual pairs.
    		for currency in wallets.currencies() {
				if currency == &base_currency {
        			let wallet = wallets.get(currency).unwrap();

					found_currency = true;
        			total_liquid = wallet.wallet_balance;
					available_liquid = wallet.available_balance;
					//debug
					//println!("{:#?}", wallet);
				}
			}
			if !found_currency {bail!("Currency needed for trade not found in wallet")}
			if available_liquid == 0.0 {bail!("Your base currency for the current pair has no free liquidity")}

			/*
			//  Taking user inputs for order
			*/

			let risk = ask("[Risk % of sub]", Some("orderrisk".to_string()))?.parse::<f64>()?;
			let stoploss = ask("[Stop-Loss]", Some("orderstoploss".to_string()))?.parse::<f64>()?;
			let takeprofit = ask("[Take-Profit]", Some("ordertakeprofit".to_string()))?.parse::<f64>()?;
			
			//get current pair price
			let q_tickers = api.fetch_tickers(pair).await?;
			let ticker = q_tickers.tickers().next().unwrap();
			let last_price = ticker.last_price.parse::<f64>()?;

			let q_symbols = api.fetch_symbols().await?;
			let symbol_info = q_symbols.symbols().into_iter().find(|symbol| symbol.alias == pair.to_owned()).unwrap();
			let min_qty = symbol_info.lot_size_filter.min_trading_qty;
			let qty_step = symbol_info.lot_size_filter.qty_step;

			//debug
			//println!("{:?}", symbol_info);

			let entry: f64;
			let istaker: bool;
			if order_type == OrderEntryType::Limit {
				entry = ask("[Entry Price]", Some("entryprice".to_string()))?.parse::<f64>()?;
				istaker = false;
			} else {
				entry = last_price;
				istaker = true;
			}

			if !((entry < takeprofit && entry > stoploss)||(entry > takeprofit && entry < stoploss)) {
				bail!("Your take-profit and stoploss parameters were not valid in comparison to your entry price. Check your parameters")
			}

			let values = misc::OrderCalcEntry {
				total_liquid,
				risk,
				stoploss,
				takeprofit,
				entry,
			};

			let calculation: misc::OrderCalcExit = misc::calculate_order(values)?;

			//real quantity calculation
			let mut qty_to_buy: f64 = 0.0;
			if pair.ends_with("USDT") {
				qty_to_buy = calculation.quantity/last_price;
			} else if pair.contains("USD")
			||pair.contains("USD0325")
			||pair.contains("USD0624") {
				qty_to_buy = last_price*calculation.quantity;
			} else {
				bail!("Your current pair is not supported for quantity calculation (devs probably lazy) notify us on our repository.");
			}
			let old_qty = qty_to_buy;

			//change quantity to buy to stepped version so bybit api can accept it
			qty_to_buy = ((qty_to_buy/qty_step).round() * qty_step);

			if qty_to_buy == 0.0 {
				bl();
				bail!("You have less than {} {}, when above {} is required. Please add more balance to your {} allocation.", qty_step/2.0, &alt_base, qty_step, &alt_base)
			} else if qty_to_buy < min_qty*10.0 {
				bl();
				println!("  {}", boldt("Warning"));
				println!("    You have a {} {} balance compared to bybits' minimum increment ({}). ",  if qty_to_buy < min_qty*5.0 {"CONSIDERABLY low"} else {"low"}, alt_base, min_qty);
				println!("    You may be risking more or less than you want in this trade.");
				bl();
			}

			/*
			//  Giving parameters to check for validity
			*/

			println!("  {}", boldt("Valid Parameters"));
			println!(
				"    Direction Type: {}",
				if calculation.islong { "Long" } else { "Short" }
			);
			println!(
				"    Trigger Type: {}",
				match order_type {
					OrderEntryType::Market => "Market",
					OrderEntryType::Limit => "Limit"
				}
			);
			println!(
				"    Order Size: {:.7} {}",
				if long_dec_needed {format!("{:.10}", last_price*qty_to_buy)} else {format!("{:.2}",  last_price*qty_to_buy)},
				&base_currency
			);
			println!(
				"    Alt Order Size: {:.7} {}",
				if !long_dec_needed {format!("{:.5}", qty_to_buy)} else {format!("{:.2}",  qty_to_buy)},
				alt_base
			);
			println!("    SL-TP Ratio : {:.2}R", calculation.tpslratio);
			println!("    L % Decrease (Stepped): {:.3}", (qty_to_buy/old_qty)*risk);
			println!("    W % Increase : {:.2}%", calculation.tpslratio*risk); //calculation.tpslratio*risk

			let fee_rate = (if istaker {symbol_info.taker_fee.to_owned()} else {symbol_info.maker_fee.to_owned()}).parse::<f64>()?;
			let entry_fees = calculation.quantity * fee_rate;

			let split_fee_win = available_liquid*(((calculation.tpslratio/100.0)*risk)-risk/100.0);
			let split_fee_percentage = entry_fees/split_fee_win;
			
			println!(
				"    Entry Fees: {:.10} {} (Split WL: {:.4}% {:.0}x)",
				if long_dec_needed {format!("{:.10}", entry_fees)} else {format!("{:.2}", entry_fees)},
				&base_currency,
				split_fee_percentage,
				100.0/(split_fee_percentage*100.0)
			);

			bl();
			println!("{}", boldt("Confirm Trade?"));
			yn(ask("(y/n)", None)?)?;

			if calculation.tpslratio < db_info.ratio_warn_num {
				bl();
				println!("{}", boldt("The SLTP ratio is not favourable. Proceed?"));
				yn(ask("(y/n)", None)?)?;
			}

			/*
			//  Using bybit API to start order
			*/

			//price setting
			let mut price_to_buy: Option<f64> = Some(entry);
			let mut bybit_rs_ot: OrderType = OrderType::Limit;

			match order_type {
				OrderEntryType::Market => {
					price_to_buy = None;
					bybit_rs_ot = OrderType::Market;
				},
				OrderEntryType::Limit => {
					//and so the useless code spread...
				},
				_ => {
					bail!("Big Panik! order_type not supported at order_type match")
				}
			}

			let order_data = PlaceActiveOrderData {
				symbol: pair.to_string(),
				side: if calculation.islong {Side::Buy} else {Side::Sell},
				qty: qty_to_buy,
				order_type: bybit_rs_ot,
				price: price_to_buy,
				time_in_force: TimeInForce::PostOnly,
				take_profit: Some(takeprofit),
				stop_loss: Some(stoploss),
				reduce_only: Some(false),
				close_on_trigger: Some(false),
				..Default::default()
			};

			//debug
			bl();
			println!("{:#?}", order_data);
			let order_id: String;
			let stop_loss: f64;
			let take_profit: f64;

			if is_linear {
				let order_result = api.place_active_linear_order(order_data).await?;
				println!("{:?}", order_result);

				order_id = order_result.id.to_string();
				stop_loss = order_result.stop_loss;
				take_profit = order_result.take_profit;
			} else {
				let order_result = api.place_active_order(order_data).await?;
				println!("{:?}", order_result);

				order_id = order_result.id.to_string();
				stop_loss = order_result.stop_loss;
				take_profit = order_result.take_profit;
			}
			
			let inserted_trade = db_insert_ftrade(db::Trade {
				_id: None,
				exchange: Exchange::Bybit,
				sub_account_name: sub_account.to_owned(),
				timestamp_open: Utc::now().timestamp().to_string().parse::<i64>()?,
				filled: false,
				risk,
				main_id: order_id,
				stop_loss: Some(stop_loss),
				take_profit: Some(take_profit),
				sl_id: None,
				tp_id: None,
			})?;
			//open database after update so no conflicts
			let mut db = Database::open(database_location().as_str()).unwrap();
			let mut collection = db.collection("ftrades").unwrap();

			bl();
			println!("{:#?}", inserted_trade);

			bl();
			println!("{}", boldt("  ORDER COMPLETE!"));

		},
		"bal"|"balance" => {
			let wallets = api.fetch_wallets().await?;

    		for currency in wallets.currencies() {
        		let wallet = wallets.get(currency).unwrap();
        		println!("  {}: {}", currency, wallet.wallet_balance);
				//debug msg
				//println!("{:#?}", wallet);
    }
		},
		//testing bybit-rs http error
		"http_error" => {
			use bybit::http::{Error as HttpError, ErrorCode};

    		let error: Error = HttpError::ErrorCode(ErrorCode {
        		code: 0,
        		msg: "".to_string(),
        		ext_code: "".to_string(),
        		ext_info: "".to_string(),
   			})
    		.into();
    		println!("{}", error);
		},
		//testing bybit-rs ws error
		"ws_error" => {
			use bybit::ws::{Channel, Error as WsError};

    		let error: Error = WsError::NotSubscribed(Channel::Insurance).into();
    		println!("{}", error);
		}
		//
		_ => (is_real_command = false),
	}
	Ok(is_real_command)
}