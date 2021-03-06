use anyhow::{bail, Error, Result,};
use bybit::{
    rest::*,
    OrderType,
    Side,
    TimeInForce,
    TriggerPrice,
    http::{self, Client as BybitClient}
};
use async_trait::async_trait;
use polodb_core::Database;
use chrono::Utc;
use tui::{backend::Backend, Terminal};
use crate::{
    Exchange,
    ForceOption,
    UserSpace,
    utils::{
        yn,
        s_or_not
    },
    db::*,
    orders::*,
    command_handling::{
        CommandHandling,
        api_check,
    }
};
use super::bybit_utils::*;

pub struct BybitStruct {
    pub bybit_api: Option<BybitClient>,
	//pub bybit_default_pair: String,
	//pub bybit_default_sub: String,
}

#[async_trait]
impl<B: Backend + std::marker::Send> CommandHandling<B> for BybitStruct {
    async fn price(&self, us: &mut UserSpace) -> Result<(), Error> {
        let bybit_api = api_check(self.bybit_api.as_ref())?;

        let q_tickers = bybit_api.fetch_tickers(&us.pair).await?;
        let ticker = q_tickers.tickers().next().force()?;
        us.prnt(format!("  Mid: {}", ticker.last_price));
        us.prnt(format!("  Ask: {}", ticker.ask_price));
        us.prnt(format!("  Bid: {}", ticker.bid_price));

        Ok(())
    }

    async fn balance(&self, us: &mut UserSpace) -> Result<(), Error> {
        let bybit_api = api_check(self.bybit_api.as_ref())?;

        let wallets = bybit_api.fetch_wallets().await?;
			let mut _more_pairs = false;

			for currency in wallets.currencies() {
				let wallet = wallets.get(currency).force()?;
				if wallet.wallet_balance > 0.0 {
					us.prnt(format!("  {}: {}", currency, wallet.wallet_balance));
				} else {
					_more_pairs = true;
				}
			}

        Ok(())
    }

    async fn order(&self, us: &mut UserSpace, terminal: &mut Terminal<B>) -> Result<(), Error> {
        let bybit_api = api_check(self.bybit_api.as_ref())?;

        let mut _order_type: OrderEntryType = OrderEntryType::Market;
        let mut _conditional_is_market = true;

        us.prnt(" Opening a trade...".to_string());

        // Function for choice command
        match us.input_old.as_str() {
            "m" => {
                _order_type = OrderEntryType::Market;
            }
            "l" => {
                _order_type = OrderEntryType::Limit;
            }
            "ob" => {
                _order_type = OrderEntryType::OrderBook;
            }
            _ => {
                let mut temp_order_type: String;
                us.prnt(" Choose a trade type option".to_string());
                loop {
                    temp_order_type = us.ask_input("m|l|ob", terminal, Some("temp_order_type")).await?;
                    match temp_order_type.as_str() {
                        "m" => {
                            _order_type = OrderEntryType::Market;
                        }
                        "l" => {
                            _order_type = OrderEntryType::Limit;
                        }
                        "ob" => {
                            _order_type = OrderEntryType::OrderBook;
                        }
                        _ => {
                            us.bl();
                            us.prnt("  !!! NOT AN OPTION: Please choose one of the options !!!".to_string());
                            us.bl();
                            continue;
                        }
                    }
                    break;
                }
            }
        }

        let base_currency = get_base(us.pair.to_owned())?;
        let alt_base: String;
        let mut long_dec_needed = true;
        let mut is_linear = true;

        if us.pair.starts_with(&base_currency) {
            alt_base = us.pair.strip_prefix(&base_currency).force()?.to_string();
            is_linear = false;
        } else {
            alt_base = us.pair.strip_suffix(&base_currency).force()?.to_string();
            long_dec_needed = false;
        }

        let mut total_liquid: f64 = 0.0;
        let mut available_liquid: f64 = 0.0;
        let mut found_currency = false;

        let wallets = bybit_api.fetch_wallets().await?;

        // This infers that risk is based on singular balance of coin. It may not spread across inverse perpetual pairs.
        for currency in wallets.currencies() {
            if currency == &base_currency {
                let wallet = wallets.get(currency).force()?;

                found_currency = true;
                total_liquid = wallet.wallet_balance;
                available_liquid = wallet.available_balance;
                //us.prnt(format!("{:?}", wallet));
            }
        }
        if !found_currency {
            bail!("Currency needed for trade not found in wallet")
        }
        if available_liquid == 0.0 {
            bail!("Your base currency for the current pair has no free liquidity")
        }

        /*
        //  Taking user inputs for order
        */

        let risk = us.ask_input("Risk % of sub", terminal, Some("order_risk")).await?.parse::<f64>()?;
        let stoploss = us.ask_input("Stop-Loss", terminal, Some("order_stoploss")).await?.parse::<f64>()?;
        let takeprofit = us.ask_input("Take-Profit", terminal, Some("order_take_profit")).await?.parse::<f64>()?;

        // Get current pair price
        let q_tickers = bybit_api.fetch_tickers(&us.pair).await?;
        let ticker = q_tickers.tickers().next().force()?;
        let last_price = ticker.last_price.parse::<f64>()?;

        let q_symbols = bybit_api.fetch_symbols().await?;
        let symbol_info = q_symbols
            .symbols()
            .into_iter()
            .find(|symbol| symbol.alias == *us.pair)
            .force()?;
        let min_qty = symbol_info.lot_size_filter.min_trading_qty;
        let qty_step = symbol_info.lot_size_filter.qty_step;

        //us.prnt(format!("{:?}", symbol_info));

        let entry: f64;
        let mut conditional_actual_entry: Option<f64> = None;
        let _is_taker: bool;
        match _order_type {
            OrderEntryType::Limit => {
                entry = us.ask_input("Entry Price", terminal, Some("entry_price")).await?.parse::<f64>()?;
                _is_taker = false;
            }
            OrderEntryType::Market => {
                entry = last_price;
                _is_taker = true;
            }
            OrderEntryType::Conditional => {
                entry = us.ask_input("Trigger Price", terminal, Some("conditional_trigger_price")).await?.parse::<f64>()?;
                if !_conditional_is_market {
                    conditional_actual_entry = Some(
                        us.ask_input("Entry Price", terminal, Some("conditional_entry_price")).await?.parse::<f64>()?,
                    );
                }
                _is_taker = _conditional_is_market;
            }
            _ => bail!("Big Panik! order_type not supported at order_type match"),
        }
        if !((entry < takeprofit && entry > stoploss)
            || (entry > takeprofit && entry < stoploss))
        {
            bail!("Your take-profit and stoploss parameters were not valid in comparison to your entry price ({entry}). Check your parameters")
        }

        let values = OrderCalcEntry {
            total_liquid,
            risk,
            stoploss,
            takeprofit,
            entry,
        };

        let calculation: OrderCalcExit = calculate_order(values)?;

        // Real quantity calculation
        let mut qty_to_buy: f64;

        if us.pair.ends_with("USDT") {
            qty_to_buy = calculation.quantity / entry;
        } else if us.pair.ends_with("USD")
            || us.pair.ends_with("USD0325")
            || us.pair.ends_with("USD0624")
        {
            qty_to_buy = entry * calculation.quantity;
        } else {
            bail!("Your current pair is not supported for quantity calculation (devs probably lazy) notify us on our repository.");
        }
        let old_qty = qty_to_buy;

        // Change quantity to buy to stepped version so bybit api can accept it
        qty_to_buy = (qty_to_buy / qty_step).round() * qty_step;

        if qty_to_buy == 0.0 {
            us.bl();
            bail!("You have less than {} {}, when above {} is required. Please add more balance to your {} allocation.", qty_step/2.0, &alt_base, qty_step, &alt_base)
        } else if qty_to_buy < min_qty * 10.0 {
            us.bl();
            us.prnt("  Warning".to_string());
            us.prnt(format!("    You have a {} {} buying availability compared to bybits' minimum increment ({}). ",  if qty_to_buy < min_qty*5.0 {"CONSIDERABLY low"} else {"low"}, alt_base, min_qty));
            us.prnt("    You may be risking more or less than you want in this trade.".to_string());
            us.bl();
        }

        //  Giving parameters to check for validity
        us.prnt("  Valid Parameters".to_string());
        us.prnt(format!(
            "    Direction Type: {}",
            if calculation.islong { "Long" } else { "Short" }
        ));
        us.prnt(format!(
            "    Trigger Type: {}",
            match _order_type {
                OrderEntryType::Market => "Market",
                OrderEntryType::Limit => "Limit",
                OrderEntryType::Conditional =>
                    if _conditional_is_market {
                        "Conditional Market"
                    } else {
                        "Conditional Limit"
                    },
                _ => bail!("Big Panik! order_type not supported at order_type match"),
            }
        ));
        us.prnt(format!(
            "    Order Size: {:.7} {}",
            if long_dec_needed {
                format!("{:.10}", entry * qty_to_buy)
            } else {
                format!("{:.2}", entry * qty_to_buy)
            },
            &base_currency
        ));
        us.prnt(format!(
            "    Alt Order Size: {:.7} {}",
            if !long_dec_needed {
                format!("{:.3}", qty_to_buy)
            } else {
                format!("{:.2}", qty_to_buy)
            },
            alt_base
        ));
        us.prnt(format!("    SL-TP Ratio: {:.2}R", calculation.tpslratio));
        us.prnt(format!("    L % Decrease: {:.3}%", (qty_to_buy / old_qty) * risk));
        us.prnt(format!(
            "    W % Increase: {:.2}%",
            (qty_to_buy / old_qty) * calculation.tpslratio * risk
        )); //calculation.tpslratio*risk

        /* Broken fees code (fix one day pls)
        let fee_rate = (if is_taker {symbol_info.taker_fee.to_owned()} else {symbol_info.maker_fee.to_owned()}).parse::<f64>()?;
        let entry_fees = (qty_to_buy/old_qty)*calculation.quantity*fee_rate;

        let split_fee_win = available_liquid*((((((calculation.tpslratio)/100.0)*risk)+risk)*2.0)/100.0);
        let split_fee_percentage = entry_fees/split_fee_win;
        */

        /*
        us.prnt(format!(
            "    Entry Fees: {:.10} {}",
            if long_dec_needed {format!("{:.10}", entry_fees)} else {format!("{:.2}", entry_fees)},
            &base_currency,
        ));

        us.prnt(format!(
            "    Split WL Fees: {:.4}% {:.0}x",
            split_fee_percentage,
            100.0/(split_fee_percentage*100.0)
        ));
        */

        us.bl();
        us.prnt("Confirm Trade".to_string());
        yn(us.ask_input("(y/n)", terminal, None).await?)?;

        if calculation.tpslratio < us.db_info.ratio_warn_num {
            us.bl();
            us.prnt("The SLTP ratio is not favourable. Proceed?".to_string());
            yn(us.ask_input("(y/n)", terminal, None).await?)?;
        }

        /*
        //  Using bybit API to start order
        */
        let order_id: String;
        let stop_loss: f64;
        let take_profit: f64;

        match _order_type {
            OrderEntryType::Market | OrderEntryType::Limit => {
                let mut price_to_buy: Option<f64> = Some(entry);
                let mut bybit_rs_ot: OrderType = OrderType::Limit;

                if _order_type == OrderEntryType::Market {
                    price_to_buy = None;
                    bybit_rs_ot = OrderType::Market;
                }

                let order_data = PlaceActiveOrderData {
                    symbol: us.pair.to_string(),
                    side: if calculation.islong {
                        Side::Buy
                    } else {
                        Side::Sell
                    },
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

                us.prnt(format!("{:?}", &order_data));

                if is_linear {
                    let order_result = bybit_api.place_active_linear_order(order_data).await?;

                    order_id = order_result.id.to_string();
                    stop_loss = order_result.stop_loss;
                    take_profit = order_result.take_profit;

                    us.prnt(format!("{:?}", order_result));

                } else {
                    let order_result = bybit_api.place_active_order(order_data).await?;

                    order_id = order_result.id.to_string();
                    stop_loss = order_result.stop_loss;
                    take_profit = order_result.take_profit;

                    us.prnt(format!("{:?}", order_result));
                }
            }
            OrderEntryType::Conditional => {
                let mut price_to_buy: Option<f64> = conditional_actual_entry;
                let mut bybit_rs_ot: OrderType = OrderType::Limit;

                if _conditional_is_market {
                    price_to_buy = None;
                    bybit_rs_ot = OrderType::Market;
                }

                let order_result = if is_linear {
                    let order_data = PlaceLinearConditionalOrderData {
                        symbol: us.pair.to_string(),
                        side: if calculation.islong {
                            Side::Buy
                        } else {
                            Side::Sell
                        },
                        qty: qty_to_buy,
                        order_type: bybit_rs_ot,
                        price: price_to_buy,
                        time_in_force: TimeInForce::PostOnly,
                        take_profit: Some(takeprofit),
                        stop_loss: Some(stoploss),
                        reduce_only: false,
                        close_on_trigger: false,
                        base_price: last_price, /* maybe change this to a more updated last price if this causes problems in the future */
                        stop_px: entry,
                        trigger_by: Some(TriggerPrice::LastPrice),
                        ..Default::default()
                    };

                    us.prnt(format!("{:?}", &order_data));

                    bybit_api.place_linear_conditional_order(order_data).await?
                } else {
                    let order_data = PlaceActiveOrderData {
                        symbol: us.pair.to_string(),
                        side: if calculation.islong {
                            Side::Buy
                        } else {
                            Side::Sell
                        },
                        qty: qty_to_buy,
                        order_type: bybit_rs_ot,
                        price: price_to_buy,
                        time_in_force: TimeInForce::PostOnly,
                        take_profit: Some(takeprofit),
                        stop_loss: Some(stoploss),
                        reduce_only: Some(false),
                        close_on_trigger: Some(false),
                        base_price: Some(last_price.to_string()), /* maybe change this to a more updated last price if this causes problems in the future */
                        stop_px: Some(entry.to_string()),
                        trigger_by: Some(TriggerPrice::LastPrice),
                        ..Default::default()
                    };

                    us.prnt(format!("{:?}", &order_data));

                    bybit_api.place_conditional_order(order_data).await?
                };

                order_id = order_result.id.to_string();
                stop_loss = order_result.stop_loss;
                take_profit = order_result.take_profit;

                us.prnt(format!("{:?}", order_result));
            }
            _ => {
                bail!("Big Panik! order_type not supported at api interaction match statement (order type not implemented yet)")
            }
        }

        let inserted_trade = db_insert_ftrade(Trade {
            _id: None,
            sub_account_name: us.sub_account.to_string(),
            timestamp_open: Utc::now().timestamp().to_string().parse::<i64>()?,
            filled: false,
            risk,
            main_id: order_id,
            stop_loss: Some(stop_loss),
            take_profit: Some(take_profit),
            sl_id: None,
            tp_id: None,
            exchange: Exchange::Bybit,
            exchange_context: if is_linear {
                ExchangeContext::BybitLinear
            } else {
                ExchangeContext::BybitInverse
            },
            entry_type: if (_order_type == OrderEntryType::Conditional && _conditional_is_market)
                || _order_type == OrderEntryType::Market
            {
                EntryType::Market
            } else {
                EntryType::Limit
            },
            client_order_type: _order_type,
            direction: if calculation.islong {
                OrderDirection::Long
            } else {
                OrderDirection::Short
            },
        })?;
        // Open database after update so no conflicts
        let mut db = Database::open(database_location().as_str())?;
        let _collection = db.collection("ftrades")?;

        us.bl();
        us.prnt(format!("{:?}", inserted_trade));

        us.bl();
        us.prnt("ORDER COMPLETE!".to_string());

        Ok(())
    }

    async fn config_defaults(&self, us: &mut UserSpace, terminal: &mut Terminal<B>) -> Result<(), Error> {
        // Temporary function
        async fn bybit_defaults<B: Backend>(us: &mut crate::UserSpace, terminal: &mut crate::Terminal<B>) -> Result<(), Error> {
            let mut database = Database::open(database_location().as_str())?;

            us.prnt("  1. Change default pair to current pair".to_string());
            us.prnt("  2. Change default subaccount to current subaccount".to_string());

            loop {
                us.bl();
                let choice = us.ask_input(
                    "Option Number",
                    terminal,
                    Some("bybit_defaults_option_number"),
                ).await?;
                match choice.as_str() {
                    "1" => {
                        //let pair_to_check = ask("", None)?;
                        us.prnt(db_insert_config(&mut database, "bybit_default_pair", &us.pair)?);
                        us.prnt(" Changed default pair successfully".to_string());
                    }
                    "2" => {
                        //let subaccount_to_check = ask("", None)?;
                        db_insert_config(&mut database, "bybit_default_sub", &us.sub_account)?;
                        us.prnt(" Changed default subaccount successfully".to_string());
                    }
                    _ => {
                        us.prnt(" !! Not a choice !!".to_string());
                        continue;
                    }
                }
                break;
            }
            Ok(())
        }

        loop {
            us.prnt(" 1. Bybit defaults".to_string());
            us.prnt(" 2. Universal defaults".to_string());
            let default_context = us.ask_input(
                "Choose an option number",
                terminal,
                Some("default_context"),
            ).await?;
            us.bl();
            match default_context.as_str() {
                "1" => {
                    us.prnt(" Bybit defaults".to_string());
                    us.bl();
                    bybit_defaults(us, terminal).await?;
                }
                "2" => {
                    us.prnt(" Universal defaults".to_string());
                    us.bl();
                    us.prnt(" Not Available yet!".to_string());
                    us.bl();
                }
                _ => {
                    us.prnt("NOT AN OPTION, CHOOSE AGAIN".to_string());
                    us.bl();
                    continue;
                }
            }
            break;
        }

        Ok(())
    }

    async fn setup_api_keys(&self, us: &mut UserSpace, terminal: &mut Terminal<B>) -> Result<(), Error> {
        let bybit_pub_key = us.ask_input("Public Bybit API key", terminal, None).await?;
        let bybit_priv_key = us.ask_input("Private Bybit API key", terminal, None).await?;

        let test_client = BybitClient::new(
			http::MAINNET_BYBIT,
			&bybit_pub_key,
			&bybit_priv_key,
		).force()?;

        let test_request = test_client.fetch_wallets().await;

        us.bl();
        match test_request {
            Ok(_x) => {
                let mut database = Database::open(database_location().as_str())?;
                db_insert_config(&mut database, "bybit_pub_key", &bybit_pub_key)?;
                db_insert_config(&mut database, "bybit_priv_key", &bybit_priv_key)?;
                us.switch_exchange(Exchange::Bybit).await?;
                us.prnt(" Successfully setup API keys for Bybit".to_string());
            },
            Err(err) => {
                us.prnt(format!(" !!! API KEYS FAILED: {} !!!", err));
            }
        }

        Ok(())
    }


    // Commands with args


    async fn search(&self, us: &mut UserSpace, terminal: &mut Terminal<B>, command: &str) -> Result<(), Error> {
        let bybit_api = api_check(self.bybit_api.as_ref())?;

        let to_search = if command.starts_with("search ") {
            command.split("search ").collect()
        } else {
            us.ask_input("Search pairs", terminal, Some("pair_search")).await?
        };
        // Grabs second part of command to search for
        let q_symbols = bybit_api.fetch_symbols().await?;
        let symbols = q_symbols.symbols();
        //us.bl();

        let mut matched_count: u32 = 0;
        // Loop over all markets
        for symbol in symbols {
            if symbol.alias.contains(&to_search.to_uppercase()) {
                //presents to user the match found
                us.prnt(format!("  HIT: {}", symbol.alias));
                //increases matched search counter
                matched_count += 1;
            }
        }
        // Gives result of search operation
        us.prnt(format!("    {} Pair{} Found", matched_count, s_or_not(matched_count as usize)));

        Ok(())
    }

    async fn change_pair(&self, us: &mut UserSpace, terminal: &mut Terminal<B>, command: &str) -> Result<(), Error> {
        let bybit_api = api_check(self.bybit_api.as_ref())?;

        let mut joined_pair = if command.starts_with("pair ") {
            command.split("pair ").collect()
        } else {
            // Accept user input in two parts
            us.prnt("  Change pair:".to_string());
            let prefix = us.ask_input("Prefix", terminal, Some("prefix_pair")).await?;
            let suffix = us.ask_input("Suffix", terminal, Some("suffix_pair")).await?;

            format!("{}{}", prefix, suffix)
        };

        joined_pair = joined_pair.to_uppercase();

        let q_symbols = bybit_api.fetch_symbols().await?;
        let mut is_real_pair: bool = false;

        for symbol in q_symbols.symbols() {
            if symbol.alias == joined_pair {
                is_real_pair = true;
            }
        }

        match is_real_pair {
            true => {
                us.prnt("    Switched (pair found)".to_string());
                let q_tickers = bybit_api.fetch_tickers(&joined_pair).await?;
                let ticker = q_tickers.tickers().next().force()?;

                // Changes global pair value to new chosen pair
                us.pair = joined_pair.to_string();
                us.prnt(format!("    Price ({}): {}", us.pair, ticker.last_price));

                us.prnt(format!("{:?}", ticker));
            }
            false => {
                us.prnt("    Switch failed (pair not found)".to_string());
            }
        }

        Ok(())
    }
}