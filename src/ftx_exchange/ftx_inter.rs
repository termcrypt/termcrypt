use ftx::{options::Options, rest::Account, rest::*};

use anyhow::{bail, Error, Result};

use dotenv::dotenv;

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

use super::super::db::*;
use super::super::misc;
use super::super::utils::{askout as ask, boldt, yn};
use super::ftx_advanced_orders::*;
use super::ftx_utils::*;

//Command Handling
pub async fn handle_commands<'a>(
	x: &str,
	subaccount: &mut String,
	pair: &mut String,
	api: &mut Rest,
	account: &mut Account,
	_iswide: bool,
	db_info: &mut super::super::Config,
) -> Result<bool, Error> {
	dotenv().ok();
	let mut isrealcommand = true;
	//handles the command given by the user
	match x {
		//lists all commands
		"h" | "help" => {
			//i would have made this dynamic but brain shite
			println!("{}", boldt("UTILITY"));
			println!("  clr | clear - clear the terminal");
			println!("  h | help - get information about commands");
			println!("  q - quits input when inside function");
			println!("  date - get current local and utc date");
			println!("  time - get current local and utc time");
			println!("  ses | sessions - get trading session times");
			println!("{}", boldt("SUBACCOUNTS"));
			println!("  subs - list all subaccounts");
			println!("  sub [nickname] - change subaccount (case sensitive)");
			println!("  bal | balance - get balances of subaccount");

			println!("{}", boldt("MARKETS"));
			println!("  search [query] - return all pairs containing query");
			println!("  p | price - return the mark, ask and bid price for current pair");
			println!("  pair - gives you an input to change the pair");
			println!("  ob | orderbook - get display of orderbook");

			println!("{}", boldt("ORDERS"));
			println!("  lev - get current account leverage");
			println!("  lev [number] - change leverage to chosen number");
			println!("  o | order - start an order");

			println!("{}", boldt("SETTINGS"));
			println!("  def | defaults - change termcrypt startup defaults");
			println!("  conf | config - change termcrypt configuration variables");

			println!("{}", boldt("KEYBINDS"));
			println!("  [UP ARROW] - Replaces input with previous command");
			println!("  [DOWN ARROW] - Replaces input with the latter command");
			println!();
			println!("  More information is available in our documentation.");
		}
		//function to make sure user does not give wrong input
		"sub" | "search" => {
			println!(
				"  !For this function, please use this format: {} [parameters]",
				x
			);
		}
		"def" | "defaults" => {
			println!("{}", boldt("AVAILABLE DEFAULTS"));
			println!("  1. Change default pair to current");
			println!("  2. Change default subaccount to current");
			let choice = ask("[OPTION NUMBER]", Some("defaultsoptionnumber".to_string()))?;
			let db: sled::Db = sled::open(database_location().as_str())?;

			match choice.as_str() {
				"1" => {
					insert_db_info_entry(&db, "default_pair", pair)?;
					println!("  Changed default pair successfully");
				}
				"2" => {
					insert_db_info_entry(&db, "default_sub", subaccount)?;
					println!("  Changed default subaccount successfully");
				}
				_ => {
					println!("  {}", boldt("!! Not a choice !!"));
				}
			}
		}
		"conf" | "config" => {
			//
		}
		//change the current pair
		x if x.starts_with("pair ") => {
			let _tosearch: String = x.split("pair ").collect();
			//TBF for specific pair change
		}
		//function to change the current subaccount in one command
		x if x.starts_with("sub ") => {
			let sub_to_search: String = x.split("sub ").collect();
			*db_info = get_db_info(false).await.unwrap();
			match sub_to_search.as_str() {
				"def" => {
					//changes to default account (not a subaccount)
					*subaccount = "def".to_string();
					println!("    {}", boldt("Returning to default account"));
					*api = Rest::new(Options {
						key: Some(db_info.ftx_pub_key.to_string()),
						secret: Some(db_info.ftx_priv_key.to_string()),
						subaccount: None,
						endpoint: ftx::options::Endpoint::Com,
					});
				}
				_ => {
					let q_subaccounts = api.request(GetSubaccounts).await?;

					let mut did_find = false;
					//searches subaccounts by nickname for user choice
					for subacc in &q_subaccounts {
						if subacc.nickname.as_str() == sub_to_search {
							//changes subaccount display variable
							*subaccount = sub_to_search.to_string();
							*api = Rest::new(Options {
								key: Some(db_info.ftx_pub_key.to_string()),
								secret: Some(db_info.ftx_priv_key.to_string()),
								subaccount: Some(sub_to_search.to_string()),
								endpoint: ftx::options::Endpoint::Com,
							});
							did_find = true;
							println!("    {}", boldt("Success (switched to subaccount)"));
						}
					}
					if !did_find {
						println!("  No subaccount found called - {}", sub_to_search);
					}
				}
			}
		}
		//change account leverage
		x if x.starts_with("lev ") => {
			let raw_lev_choice: String = x.split("lev ").collect();
			let lev_choice: u32 = raw_lev_choice.parse::<u32>()?;

			let q_account = api.request(GetAccount).await?;
			api.request(ChangeAccountLeverage::new(lev_choice)).await?;

			println!("CHANGE: {} -> {}", q_account.leverage, lev_choice);
			println!("  {}", boldt("Success (changed leverage)"));
		}
		//search for market by query
		x if x.starts_with("search ") => {
			//grabs second part of command to search for
			let to_search: String = x.split("search ").collect();
			let markets = api.request(GetMarkets).await?;
			println!();
			let mut matched_count: i32 = 0;
			//loop over all markets
			for market in &markets {
				if market.name.contains(&to_search.to_uppercase()) {
					//presents to user the match found
					println!("HIT: {}", market.name);
					//increases matched search counter
					matched_count += 1;
				}
			}
			//gives result of search operation
			println!("  {} {}", matched_count, boldt("Pairs Found"));
		}
		//show orderbook for current market
		"ob" | "orderbook" => {
			let q_orderbook = api
				.request(GetOrderBook {
					market_name: pair.to_string(),
					depth: Some(10),
				})
				.await?;
			//println!("{:#?}", q_orderbook);

			let mut bid_width: Decimal = dec!(0);
			let mut ask_width: Decimal = dec!(0);

			for x in &q_orderbook.bids {
				let ol_length = Decimal::from_usize(x.0.to_string().len()).unwrap()
					+ Decimal::from_usize(x.1.to_string().len()).unwrap();
				if ol_length > bid_width {
					bid_width = ol_length
				};
			}

			for x in &q_orderbook.asks {
				let ol_length = Decimal::from_usize(x.0.to_string().len()).unwrap()
					+ Decimal::from_usize(x.1.to_string().len()).unwrap();
				if ol_length > ask_width {
					ask_width = ol_length
				};
			}

			let f_bid_width: Decimal = (bid_width - dec!(3)) + dec!(3).round_dp(0);
			if f_bid_width < dec!(0) {
				bid_width = dec!(0)
			};
			let f_ask_width: Decimal = (ask_width - dec!(0)) + dec!(3).round_dp(0);
			if f_ask_width < dec!(0) {
				ask_width = dec!(0)
			};

			println!(
				"{}{}",
				" ".repeat((bid_width / dec!(2)).to_usize().unwrap()),
				boldt(format!("{} {}", "ORDERBOOK FOR", pair).as_str())
			);

			println!(
				"{} BID {} {} ASK {}",
				" ".repeat((bid_width / dec!(2)).to_usize().unwrap()),
				" ".repeat((bid_width / dec!(2)).to_usize().unwrap()),
				" ".repeat((ask_width / dec!(2)).to_usize().unwrap()),
				" ".repeat((ask_width / dec!(2)).to_usize().unwrap())
			);

			for (iters, _x) in q_orderbook.asks.iter().enumerate() {
				let mut ob_line_bids = format!(
					"{} [{}]",
					q_orderbook.bids[iters].0, q_orderbook.bids[iters].1
				);
				let ob_line_width = Decimal::from_usize(ob_line_bids.len()).unwrap();
				if ob_line_width < bid_width + dec!(3) {
					ob_line_bids = format!(
						"{}{}",
						ob_line_bids,
						" ".repeat((bid_width + dec!(3) - ob_line_width).to_usize().unwrap())
					)
				};

				let ob_line_asks = format!(
					"{} [{}]",
					q_orderbook.asks[iters].0, q_orderbook.asks[iters].1
				);
				println!(" {} | {}", ob_line_bids, ob_line_asks);
			}
		}
		//initiate a market order
		"o" | "order" => {
			let q_account = api.request(GetAccount).await?;
			let q_market = api.request(GetMarket::new(pair.as_str())).await?;

			let mut total_liquid: Decimal = dec!(0);
			let mut available_liquid: Decimal = dec!(0);
			let mut found_currency = false;

			let quote_currency: String;
			let mut isfuture: bool = false;

			if pair.ends_with("PERP") | pair.ends_with("1231") {
				quote_currency = "USD".to_string();
				isfuture = true;
			} else {
				quote_currency = q_market.quote_currency.unwrap();
			}

			match subaccount.as_str() {
				"def" => {
					let q_balances = api.request(GetWalletBalances).await?;
					for balance in q_balances {
						if quote_currency == balance.coin {
							found_currency = true;
							available_liquid = balance.free;
							total_liquid = balance.total;
						}
					}
				}
				_ => {
					let q_balances = api.request(GetSubaccountBalances::new(subaccount)).await?;
					for balance in q_balances {
						if quote_currency == balance.coin {
							found_currency = true;
							available_liquid = balance.free;
							total_liquid = balance.total;
						}
					}
				}
			}

			if !found_currency {
				bail!("You have no base currency matching the current pair")
			}

			if available_liquid == dec!(0) {
				bail!("Your base currency for the current pair has no free liquidity")
			}

			let risk = ask("[Risk % of sub]", Some("orderrisk".to_string()))?.parse::<Decimal>()?;
			let stoploss =
				ask("[Stop-Loss]", Some("orderstoploss".to_string()))?.parse::<Decimal>()?;
			let takeprofit =
				ask("[Take-Profit]", Some("ordertakeprofit".to_string()))?.parse::<Decimal>()?;
			println!("    Bid: {}", q_market.bid);
			println!("    Ask: {}", q_market.ask);
			let entry_text: String = ask("[Entry | m | ob]", Some("ordertype".to_string()))?;
			let entry;

			let mut ismarket = false;
			let mut isorderbook = false;
			let mut order_book_pos: Decimal = dec!(5);

			if entry_text.to_uppercase() == *"M" {
				entry = q_market.price;
				ismarket = true;
			} else if entry_text.to_uppercase() == *"OB" {
				order_book_pos = ask(
					"[OrderBook Pos (0-9)]",
					Some("orderorderbookposition".to_string()),
				)?
				.parse::<Decimal>()?;
				isorderbook = true;

				//temporary entry price until confirmation
				entry = q_market.price;
			} else {
				entry = entry_text.parse::<Decimal>()?;
			}

			if (q_account.leverage * risk) > dec!(100) {
				bail!("You are at risk of liquidation so the trade cannot take place. Check leverage and risk.");
			}

			let values = misc::OrderCalcEntry {
				total_liquid,
				risk,
				stoploss,
				takeprofit,
				entry,
			};

			//println!("{:#?}", q_account);
			//println!("{:#?}", q_market);

			let calculation: misc::OrderCalcExit = misc::calculate_order(values)?;

			if !calculation.islong && !isfuture {
				bail!("You cannot short while not being in a future pair");
			}

			let percentage_of_liquid = ((calculation.quantity
				/ (total_liquid
					* if isfuture {
						q_account.leverage
					} else {
						dec!(1)
					})) * dec!(100))
			.round_dp(2);

			if percentage_of_liquid > dec!(100) {
				bail!("You do not have enough liquidity in your account for this trade.")
			}

			println!();
			println!("  {}", boldt("Valid Parameters"));
			if isfuture {
				println!("    Leverage: {}", q_account.leverage)
			};
			println!(
				"    Direction Type: {}",
				if calculation.islong { "Long" } else { "Short" }
			);
			println!(
				"    Market Type: {}",
				if isfuture { "Future" } else { "Spot" }
			);
			println!(
				"    Trigger Type: {}",
				if ismarket { "Market" } else { "Limit" }
			);
			if isorderbook {
				println!("    OrderBook Position: {}", order_book_pos)
			};
			println!();
			println!(
				"    Order Size: {} {}",
				calculation.quantity.round_dp(6),
				&quote_currency
			);
			println!("    SL-TP Ratio : {}R", calculation.tpslratio.round_dp(2));
			println!(
				"    % Of ({}) {} Liquidity: {}",
				subaccount, &quote_currency, percentage_of_liquid,
			);

			let fees = calculate_fees(ismarket, calculation.quantity, account);

			let fees_of_sub = fees / total_liquid;

			println!(
				"    Entry Fees: {} {} ({}% of sub)",
				fees,
				&quote_currency,
				fees_of_sub.round_dp(6)
			);
			println!();

			println!("{}", boldt("Confirm Values?"));
			yn(ask("(y/n)", None)?)?;

			if calculation.tpslratio < dec!(1) {
				println!();
				println!("{}", boldt("The SLTP ratio is not favourable. Proceed?"));
				yn(ask("(y/n)", None)?)?;
			}

			//start of ordering process

			//MAIN ORDER
			let q_main_order = o_now_order(
				NowOrder {
					pair: pair.to_string(),
					islong: calculation.islong,
					real_quantity: calculation.quantity / q_market.price,
					ismarket,
					entry: Some(entry),
					price: q_market.price,
					isorderbook,
					orderbookpos: if isorderbook {
						Some(order_book_pos)
					} else {
						None
					},
				},
				api,
			)
			.await?;

			println!("  main order id: {}", q_main_order.id);
			println!("  main order type: {:?}", q_main_order.r#type);
			println!();

			//STOPLOSS ORDER
			println!("{}", boldt("Stoploss options"));
			let sl_type;
			let sl_ismarket: bool;
			loop {
				let sl_type_in = ask("SL [m]", Some("orderstoplosstype".to_string()))?;
				match sl_type_in.to_uppercase().as_str() {
					"M" => {
						sl_type = SLType::M;
						sl_ismarket = true;
						break;
					}
					"Hs" => {
						sl_type = SLType::Hs;
						sl_ismarket = false;
						break;
					}
					_ => {
						println!("You must choose a stoploss type");
						continue;
					}
				}
			}

			let q_stop_order = o_sl_order(
				SLOrder {
					pair: pair.to_string(),
					islong: calculation.islong,
					real_quantity: calculation.quantity / q_market.price,
					stop_price: stoploss,
					sl_type,
				},
				api,
			)
			.await?;

			println!();
			println!("  Stop order id: {}", q_stop_order.id);
			println!();

			//TAKE-PROFIT ORDER
			println!("{}", boldt("Take-profit options"));
			let tp_type;
			let tp_ismarket;
			loop {
				let tp_type_in = ask("TP [m]", Some("ordertakeprofittype".to_string()))?;
				match tp_type_in.to_uppercase().as_str() {
					"M" => {
						tp_type = TPType::M;
						tp_ismarket = true;
						break;
					}
					"ob" => {
						tp_type = TPType::Ob;
						tp_ismarket = false;
						break;
					}
					_ => {
						println!("Err: You must choose a take-profit type!");
						continue;
					}
				}
			}

			let q_tp_order = o_tp_order(
				TPOrder {
					pair: pair.to_string(),
					islong: calculation.islong,
					real_quantity: calculation.quantity / q_market.price,
					tp_price: takeprofit,
					tp_type,
				},
				api,
			)
			.await?;

			let sl_fees = calculate_fees(sl_ismarket, calculation.quantity, account);
			let tp_fees = calculate_fees(tp_ismarket, calculation.quantity, account);

			println!();
			println!("  Take-profit order id: {}", q_tp_order.id);
			println!();
			println!(
				"  SL Fees: {} {} ({}% of sub)",
				sl_fees,
				&quote_currency,
				(sl_fees / total_liquid).round_dp(6)
			);
			println!(
				"  TP Fees: {} {} ({}% of sub)",
				tp_fees,
				&quote_currency,
				(tp_fees / total_liquid).round_dp(6)
			);
			let splitfees = (sl_fees / dec!(2)) + (tp_fees / dec!(2));
			println!(
				"  Split TPSL Fees: {} {} ({}% of sub)",
				splitfees,
				&quote_currency,
				(splitfees / total_liquid).round_dp(6),
			);
			println!();
			println!("{}", boldt("  ORDER COMPLETE!"));
		}
		//stops a trade
		"stop" => {
			//TBF
			//stops order
		}
		//gets current account leverage
		"lev" => {
			let q_account = api.request(GetAccount).await?;
			println!("  Current Leverage: {}", q_account.leverage);
		}
		//lists all subaccounts
		"subs" => {
			let q_subaccounts = api.request(GetSubaccounts).await?;

			let mut sub_counter: i32 = 0;
			for sub_acc in &q_subaccounts {
				sub_counter += 1;
				println!("{}. {}", sub_counter, boldt(&sub_acc.nickname));
			}
		}
		//changes the current pair
		"pair" => {
			let temp_pair: String;
			//take input for auto mode
			//accept user input in two parts
			println!("  Change pair:");
			let prefix = ask("[Prefix]", Some("prefixpair".to_string()))?;
			let suffix = ask("[Suffix]", Some("suffixpair".to_string()))?;

			//format parts into temp_pair
			temp_pair = ftx_formattedpair([prefix.as_str(), suffix.as_str()]);

			let q_markets = api.request(GetMarkets).await?;
			let mut isrealpair: bool = false;

			for market in &q_markets {
				if market.name == temp_pair.as_str() {
					isrealpair = true;
				}
			}

			match isrealpair {
				true => {
					println!("    {}", boldt("Success (pair found)"));
					let q_market = api.request(GetMarket::new(temp_pair.as_str())).await?;

					//changes pair value to new chosen pair
					*pair = temp_pair;
					println!(
						"    Price ({}): {}{}",
						pair,
						q_market.price,
						ftx_getsuffixsymbol(pair.as_str())
					);
				}
				false => {
					println!("    {}", boldt("Error (pair not found)"));
				}
			}
		}
		//gets the price of the current pair
		"p" | "price" => {
			let q_market = api.request(GetMarket::new(pair.as_str())).await?;
			println!("  Mid: {}", q_market.price);
			println!("  Ask: {}", q_market.ask);
			println!("  Bid: {}", q_market.bid);
		}
		//gets balance of current subaccount
		"bal" | "balance" => {
			match subaccount.as_str() {
				//default account (no subaccount chosen)
				"def" => {
					let q_balances = api.request(GetWalletBalances).await?;
					println!("[{} Balance types]", q_balances.len());
					for balance in &q_balances {
						println!("  {}", boldt(&balance.coin));
						println!(
							"     Free:  {} ({}%)",
							&balance.free,
							if balance.total > dec!(0) {
								(balance.free / balance.total) * dec!(100)
							} else {
								dec!(0)
							}
						);
						println!("     Total: {}", &balance.total);
						println!();
					}
				}
				//other cases (subaccount chosen)
				_ => {
					let q_balances = api
						.request(GetSubaccountBalances {
							nickname: subaccount.to_string(),
						})
						.await?;
					println!("[{} Balance types]", q_balances.len());
					for balance in q_balances {
						println!("  {}", boldt(&balance.coin));
						println!(
							"     Free:  {} ({}%)",
							&balance.free,
							if balance.total > dec!(0) {
								(balance.free / balance.total) * dec!(100)
							} else {
								dec!(0)
							}
						);
						println!("     Total: {}", &balance.total);
						println!();
					}
				}
			}
			//TBF
		}
		//gets list of all markets (including futures)
		"allmarkets" => {
			let q_markets = api.request(GetMarkets).await?;
			for market in &q_markets {
				print!("{} | ", market.name)
			}
		}
		//gets list of all futures
		"allfutures" => {
			let q_futures = api.request(GetFutures).await?;
			for future in &q_futures {
				print!("{} | ", future.name)
			}
		}
		//gets account object
		"account" => {
			*account = api.request(GetAccount).await?;
			println!("{:#?}", account);
		}
		//gets raw markets object
		"rawmarkets" => {
			println!("{:#?}", api.request(GetMarkets).await?);
		}
		_ => (isrealcommand = false),
	}
	Ok(isrealcommand)
}

pub fn calculate_fees(ismarket: bool, quantity: Decimal, account: &mut Account) -> Decimal {
	match ismarket {
		true => (quantity * account.taker_fee).round_dp(3),
		false => (quantity * account.maker_fee).round_dp(3),
	}
}
