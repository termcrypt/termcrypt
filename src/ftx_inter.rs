use ftx::{
    rest::Rest,
    options::Options,
    rest::Account
};

use anyhow::{
    Result,
    Error,
    bail
};

use dotenv::dotenv;

use rust_decimal::prelude::*;
use rust_decimal_macros::{
    dec
};

use super::advanced_orders::*;

use super::utils::{
    askout as ask,
    boldt as boldt,
    formattedpair,
    getsuffixsymbol
};

pub struct FtxHcStruct {
    pub pair: String,
    pub subaccount: String
}

//Command Handling
pub async fn handle_commands<'a>(x:&str, subaccount:&mut String, pair:&mut String, api:&mut Rest, account:&mut Account, _iswide:bool) -> Result<FtxHcStruct, Error> {
    dotenv().ok();
    //handles the command given by the user
    match x {
        //lists all commands
        "h"|"help" => {
            //i would have made this dynamic but brain shite
            println!("{}", boldt("UTILITY"));
            println!("  clr | clear - clear the terminal");
            println!("  h | help - get information about commands");
            println!("  qq - quits function (when inside function input)");

            println!("{}", boldt("SUBACCOUNTS"));
            println!("  subs - list all subaccounts");
            println!("  sub [nickname] - change subaccount (case sensitive)");
            println!("  bal | balance - get balances of subaccount");

            println!("{}", boldt("MARKETS"));
            println!("  search [query] - return all pairs containing query");
            println!("  p | price - return the mark, ask and bid price for current pair");
            println!("  pair - gives you an input to change the pair");

            println!("{}", boldt("ORDERS"));
            println!("  lev - get current account leverage");
            println!("  lev [number] - change leverage to chosen number");
            println!("  o | order - start an order");
            println!("  ob | orderbook - get display of orderbook");

            println!("{}", boldt("KEYBINDS"));
            println!("  [UP ARROW] - Replaces input with previous command");
            println!("  [DOWN ARROW] - Replaces input with the latter command")
        },
        //function to make sure user does not give wrong input
        "sub"|"search" => {
            println!("  !For this function, please use this format: {} [parameters]", x);
        },
        //change the current pair
        x if x.starts_with("pair ") => {
            let _tosearch:String = x.split("pair ").collect();
            //TBF for specific pair change
        },
        //function to change the current subaccount in one command
        x if x.starts_with("sub ") => {
            let sub_to_search:String = x.split("sub ").collect();
            match sub_to_search.as_str() {
                "def" => {
                    //changes to default account (not a subaccount)
                    *subaccount = "def".to_string();
                    println!("    {}", boldt("Returning to default account"));
                    let mut options_to_change = Options::from_env();
                    options_to_change.subaccount = None;
                    *api = Rest::new(options_to_change);
                },
                _ => {
                    let q_subaccounts = api.get_subaccounts().await?;

                    let mut did_find = false;
                    //searches subaccounts by nickname for user choice
                    for subacc in &q_subaccounts{
                        if subacc.nickname.as_str() == sub_to_search {
                            //changes subaccount display variable
                            *subaccount = sub_to_search.to_string();

                            //changes rest subaccount so ftx recieves the new subbaccount
                            let mut options_to_change = Options::from_env();
                            options_to_change.subaccount = Some(sub_to_search.to_string());
                            *api = Rest::new(options_to_change);

                            did_find = true;
                            println!("    {}", boldt("Success (switched to subaccount)"));
                        }
                    }
                    if !did_find {
                        println!("  No subaccount found called - {}", sub_to_search);
                    }
                }
            }
        },
        //change account leverage
        x if x.starts_with("lev ") => {
            let raw_lev_choice:String = x.split("lev ").collect();
            let lev_choice:i32 = raw_lev_choice.parse::<i32>()?;

            let q_account = api.get_account().await?;
            api.change_account_leverage(lev_choice).await?;

            println!("CHANGE: {} -> {}", q_account.leverage, lev_choice);
            println!("  {}", boldt("Success (changed leverage)"));
        },
        //search for market by query
        x if x.starts_with("search ") => {
            
            //grabs second part of command to search for
            let to_search:String = x.split("search ").collect();
            let markets = api.get_markets().await?;
            println!();
            let mut matched_count:i32 = 0;
            //loop over all markets
            for item in &markets {
                if item.name.contains(&to_search.to_uppercase()) {
                    //presents to user the match found
                    println!("HIT: {}", item.name);
                    //increases matched search counter
                    matched_count += 1;
                }
            }
            //gives result of search operation
            println!("  {} {}", matched_count, boldt("Pairs Found"));
        },
        //test order placement
        "test" => {
            //use usd_amount/price
            let bruh = api.place_trigger_order(
                "BTC/USD",
                ftx::rest::Side::Buy,
                dec!(20),
                ftx::rest::OrderType::Stop,
                dec!(50000),
                None,
                None,
                None,
                None
            ).await?;

            println!("{:#?}", bruh);
        },
        //show orderbook for current market
        "ob"|"orderbook" => {

            let q_orderbook = api.get_orderbook(&pair.as_str(), Some(10)).await?;
            //println!("{:#?}", q_orderbook);

            let mut bid_width:Decimal = dec!(0);
            let mut ask_width:Decimal = dec!(0);

            for x in &q_orderbook.bids {
                let ol_length = Decimal::from_usize(x.0.to_string().len()).unwrap()+Decimal::from_usize(x.1.to_string().len()).unwrap();
                if ol_length > bid_width {bid_width = ol_length};
            }

            for x in &q_orderbook.asks {
                let ol_length = Decimal::from_usize(x.0.to_string().len()).unwrap()+Decimal::from_usize(x.1.to_string().len()).unwrap();
                if ol_length > ask_width {ask_width = ol_length};
            }

            let f_bid_width:Decimal = (bid_width - dec!(3))+dec!(3).round_dp(0);
            if f_bid_width < dec!(0) {bid_width = dec!(0)};
            let f_ask_width:Decimal = (ask_width - dec!(0))+dec!(3).round_dp(0);
            if f_ask_width < dec!(0) {ask_width = dec!(0)};

            println!("{}{}", " ".repeat((bid_width/dec!(2)).to_usize().unwrap()), boldt(format!("{} {}", "ORDERBOOK FOR", pair).as_str()));

            println!("{} BID {} {} ASK {}", " ".repeat((bid_width/dec!(2)).to_usize().unwrap()), " ".repeat((bid_width/dec!(2)).to_usize().unwrap()), " ".repeat((ask_width/dec!(2)).to_usize().unwrap()), " ".repeat((ask_width/dec!(2)).to_usize().unwrap()));

            let mut iters = 0;
            for _x in &q_orderbook.asks {
                let mut ob_line_bids = format!("{} [{}]", q_orderbook.bids[iters].0, q_orderbook.bids[iters].1);
                let ob_line_width = Decimal::from_usize(ob_line_bids.len()).unwrap();
                if ob_line_width < bid_width+dec!(3) {
                    ob_line_bids = format!("{}{}", ob_line_bids, " ".repeat((bid_width+dec!(3)-ob_line_width).to_usize().unwrap()))
                };

                let ob_line_asks = format!("{} [{}]", q_orderbook.asks[iters].0, q_orderbook.asks[iters].1);
                println!(" {} | {}", ob_line_bids, ob_line_asks);
                iters += 1;
            }
        },
        //initiate a market order
        "o"|"order" => {
            let q_account = api.get_account().await?;
            let q_market = api.get_market(&pair.as_str()).await?;

            let mut total_liquid:Decimal = dec!(0);
            let mut available_liquid:Decimal = dec!(0);
            let mut found_currency = false;

            let quote_currency:String;
            let mut isfuture:bool = false;

            if pair.ends_with("PERP") | pair.ends_with("1231") {
                quote_currency = "USD".to_string();
                isfuture = true;
            } else {
                quote_currency = q_market.quote_currency.unwrap();
            }

            match subaccount.as_str() {
                "def" => {
                    let q_balances = api.get_wallet_balances().await?;
                    for balance in q_balances {
                        if quote_currency == balance.coin {
                            found_currency = true;
                            available_liquid = balance.free;
                            total_liquid = balance.total;
                        }
                    }
                },
                _ => {
                    let q_balances = api.get_subaccount_balances(subaccount).await?;
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

            let risk = ask("[Risk % of sub]")?.parse::<Decimal>()?;
            let stoploss = ask("[Stop-Loss]")?.parse::<Decimal>()?;
            let takeprofit = ask("[Take-Profit]")?.parse::<Decimal>()?;
            println!("    Bid: {}", q_market.bid);
            println!("    Ask: {}", q_market.ask);
            let entry_text:String = ask("[Entry | m | ob]")?;
            let entry;

            let mut ismarket = false;
            let mut isorderbook = false;
            let mut order_book_pos:Decimal = dec!(5);

            if entry_text.to_uppercase() == "M".to_string() {
                entry = q_market.price;
                ismarket = true;

            } else if entry_text.to_uppercase() == "OB".to_string() {
                order_book_pos = ask("[OrderBook Pos (0-9)]")?.parse::<Decimal>()?;
                isorderbook = true;

                //temporary entry price until confirmation
                entry = q_market.price;
            } else {
                entry = entry_text.parse::<Decimal>()?;
            }

            if (q_account.leverage * risk) > dec!(100) {
                bail!("You are at risk of liquidation so the trade cannot take place. Check leverage and risk.");
            }

            let values = super::misc::OrderCalcEntry {
                totalliquid: total_liquid,
                risk: risk,
                stoploss: stoploss,
                takeprofit: takeprofit,
                entry: entry
            };

            //println!("{:#?}", q_account);
            //println!("{:#?}", q_market);

            let calculation:super::misc::OrderCalcExit = super::misc::calculate_order(values)?;

            if !calculation.islong && !isfuture {
                bail!("You cannot short while not being in a future pair");
            }

            println!();
            println!("  {}", boldt("Valid Parameters"));
            if isfuture {println!("    Leverage: {}", q_account.leverage)};
            println!("    Direction Type: {}", if calculation.islong {"Long"} else {"Short"});
            println!("    Market Type: {}", if isfuture {"Future"} else {"Spot"});
            println!("    Trigger Type: {}", if ismarket {"Market"} else {"Not Market"});
            if isorderbook {println!("    OrderBook Position: {}", order_book_pos)};
            println!();
            println!("    Order Size: {} {}", calculation.quantity.round_dp(6), &quote_currency);
            println!("    SL-TP Ratio : {}R", calculation.tpslratio.round_dp(2));
            println!("    % Of ({}) {} Liquidity: {}", subaccount, &quote_currency, ((calculation.quantity / (total_liquid * if isfuture {q_account.leverage} else {dec!(1)})) * dec!(100)).round_dp(2));

            let fees;
            match ismarket {
                true => {
                    fees = (calculation.quantity * account.taker_fee).round_dp(3);
                },
                false => {
                    fees = (calculation.quantity * account.maker_fee).round_dp(3);
                }
            };

            let fees_of_sub = fees/total_liquid;

            println!("    Fees: {} {} ({}% of sub)",  fees, &quote_currency, fees_of_sub.round_dp(5));
            println!();

            println!("{}", boldt("Confirm Values?"));
            match ask("(y/n)")?.as_str() {
                "y"|"yes"|"Y"|"YES" => {
                    println!("  HIT: Confirmed");
                },
                _ => {
                    bail!("User stopped");
                }
            }

            //start of ordering process
            //main order

            let q_main_order = o_now_order(NowOrder {
                pair: pair.to_string(),
                islong: calculation.islong,
                real_quantity: calculation.quantity/q_market.price,
                ismarket: ismarket,
                entry: Some(entry),
                price: q_market.price,
                isorderbook: isorderbook,
                orderbookpos: if isorderbook {Some(order_book_pos)} else {None}
            }, api).await?;

            println!("{:#?}", q_main_order);

            //take profit and stop loss
        },
        //stops a trade
        "stop" => {
            //TBF
            //stops order
        },
        //gets current account leverage
        "lev" => {
            let q_account = api.get_account().await?;
            println!("  Current Leverage: {}", q_account.leverage);
        },
        //lists all subaccounts
        "subs" => {
            let q_subaccounts = api.get_subaccounts().await?;

            let mut sub_counter:i32 = 0;
            for sub_acc in &q_subaccounts {
                sub_counter += 1;
                println!("{}. {}", sub_counter, boldt(&sub_acc.nickname));
            }
        },
        //changes the current pair
        "pair" => {
            let temp_pair:String;
            //take input for auto mode
            //accept user input in two parts
            println!("  Change pair:");
            let prefix = ask("[Prefix]")?;
            let suffix = ask("[Suffix]")?;

            //format parts into temp_pair
            temp_pair = formattedpair([prefix.as_str(), suffix.as_str()]);
               

            let q_markets = api.get_markets().await?;
            let mut isrealpair:bool = false;
        
            for item in &q_markets {
                if item.name == temp_pair.as_str() {
                    isrealpair = true;
                }
            }
                    
            match isrealpair {
                true => {
                    println!("    {}", boldt("Success (pair found)"));
                    //gets price of pair
                    let q_market = api.get_market(&temp_pair.as_str()).await?;

                    //changes pair value to new chosen pair
                    *pair = temp_pair;
                    println!("    Price ({}): {}{}", pair, q_market.price, getsuffixsymbol(pair.as_str()));
                },
                false => {
                    println!("    {}", boldt("Error (pair not found)"));
                }
            }
        },
        //gets the price of the current pair
        "p"|"price" => {     
            let q_market = api.get_market(&pair.as_str()).await?;
            println!("  Mid: {}", q_market.price);
            println!("  Ask: {}", q_market.ask);
            println!("  Bid: {}", q_market.bid);
        },
        //gets balance of current subaccount
        "bal"|"balance" => {
            match subaccount.as_str() {
                //default account (no subaccount chosen)
                "def" => {
                    let q_balances = api.get_wallet_balances().await?;
                    println!("[{} Balance types]", q_balances.len());
                    for balance in &q_balances {
                        println!("  {}", boldt(&balance.coin));
                        println!("     Free:  {} ({})", &balance.free, (&balance.free/&balance.total)*dec!(100));
                        println!("     Total: {}", &balance.total);
                        println!();
                    }
                },
                //other cases (subaccount chosen)
                _ => {
                    let q_balances = api.get_subaccount_balances(subaccount).await?;
                    println!("[{} Balance types]", q_balances.len());
                    for balance in q_balances {
                        println!("  {}", boldt(&balance.coin));
                        println!("     Free:  {}", &balance.free);
                        println!("     Total: {}", &balance.total);
                        println!();
                    }
                }
            }
            //TBF
        },
        //gets list of all markets (including futures)
        "allmarkets" => {
            let q_markets = api.get_markets().await?;
            for item in &q_markets {
                print!("{} | ", item.name)
            }
        },
        //gets list of all futures
        "allfutures" => {
            let q_futures = api.get_futures().await?;
            for item in &q_futures {
                print!("{} | ", item.name)
            }
        },
        //gets list of all markets (including futures) and prices
        "listprices" => {
            let q_markets = api.get_markets().await?;
            for item in &q_markets {
                println!("{} - {}", item.name, item.price)
            }
        }
        //gets account object
        "account" => {
            *account = api.get_account().await?;
            println!("{:#?}", account);
        },
        //gets raw markets object
        "rawmarkets" => {
            println!("{:#?}", api.get_markets().await?);
        }
        _ => ()
    }
    return Ok (
        FtxHcStruct {
            pair: pair.to_string(),
            subaccount: subaccount.to_string()
        }
    );
}

/* SPINNER DEMO

let sp = Spinner::new(&Spinners::Line, "Searching for pair".into());

sp.stop();

*/