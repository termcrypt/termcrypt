use ftx::{
    rest::Rest,
    options::Options,
    rest::Account
};
//use ftx::rest::Error as ftxerror;
//use ParseFloatError as parsefloaterror;
//use spinners;
//use spinners::{Spinner, Spinners};
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
        "h"|"help" => {
            //displays all commands
            //i would have made this dynamic 
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

            println!("{}", boldt("KEYBINDS"));
            println!("  [UP ARROW] - Replaces input with previous command");
            println!("  [DOWN ARROW] - Replaces input with the latter command")
        },
        //function to make sure user does not give wrong input
        "sub"|"search" => {
            println!("  !For this function, please use this format: {} [parameters]", x);
        },
        //function to change the current pair in one command
        x if x.starts_with("pair ") => {
            let _tosearch:String = x.split("pair ").collect();
            //TBF
        },
        //function to change the current subaccount in one command
        x if x.starts_with("sub ") => {
            let subtosearch:String = x.split("sub ").collect();
            match subtosearch.as_str() {
                "def" => {
                    //changes to default account (not a subaccount)
                    *subaccount = "def".to_string();
                    println!("    {}", boldt("Returning to default account"));
                },
                _ => {
                    let q_subaccounts = api.get_subaccounts().await?;

                    let mut didfind = false;
                    //searches subaccounts by nickname for user choice
                    for subacc in &q_subaccounts{
                        if subacc.nickname.as_str() == subtosearch {
                            //changes subaccount display variable
                            *subaccount = subtosearch.to_string();

                            //changes rest subaccount so ftx recieves the new subbaccount
                            let mut optionstochange = Options::from_env();
                            optionstochange.subaccount = Some(subtosearch.to_string());
                            *api = Rest::new(optionstochange);

                            didfind = true;
                            println!("    {}", boldt("Success (switched to subaccount)"));
                        }
                    }
                    if !didfind {
                        println!("  No subaccount found called - {}", subtosearch);
                    }
                }
            }
        },
        x if x.starts_with("lev ") => {
            let raw_levchoice:String = x.split("lev ").collect();
            let levchoice:i32 = raw_levchoice.parse::<i32>()?;

            let q_account = api.get_account().await?;
            api.change_account_leverage(levchoice).await?;

            println!("CHANGE: {} -> {}", q_account.leverage, levchoice);
            println!("  {}", boldt("Success (changed leverage)"));
        },
        x if x.starts_with("search ") => {
            
            //grabs second part of command to search for
            let tosearch:String = x.split("search ").collect();
            let markets = api.get_markets().await?;
            println!();
            let mut matched_count:i32 = 0;
            //loop over all markets
            for item in &markets {
                if item.name.contains(&tosearch.to_uppercase()) {
                    //presents to user the match found
                    println!("HIT: {}", item.name);
                    //increases matched search counter
                    matched_count += 1;
                }
            }
            //gives result of search operation
            println!("  {} {}", matched_count, boldt("Pairs Found"));
        },
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
        "o"|"order" => {
            let q_account = api.get_account().await?;
            let q_market = api.get_market(&pair.as_str()).await?;

            let mut totalliquid:Decimal = dec!(0);
            let mut availableliquid:Decimal = dec!(0);
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
                            availableliquid = balance.free;
                            totalliquid = balance.total;
                        }
                    }
                },
                _ => {
                    let q_balances = api.get_subaccount_balances(subaccount).await?;
                    for balance in q_balances {
                        if quote_currency == balance.coin {
                            found_currency = true;
                            availableliquid = balance.free;
                            totalliquid = balance.total;
                        }
                    }
                }
            }

            if !found_currency {
                bail!("You have no base currency matching the current pair")
            }

            if availableliquid == dec!(0) {
                bail!("Your base currency for the current pair has no free liquidity")
            }

            let risk = ask("[Risk % of sub]")?.parse::<Decimal>()?;
            let stoploss = ask("[Stop-Loss]")?.parse::<Decimal>()?;
            let takeprofit = ask("[Take-Profit]")?.parse::<Decimal>()?;
            println!("    Mid: {}", q_market.price);
            println!("    Ask: {}", q_market.ask);
            println!("    Bid: {}", q_market.bid);
            let entrytext:String = ask("[Entry | m]")?;
            let entry;

            let mut ismarket = false;

            if entrytext.to_uppercase() == "M".to_string() {
                entry = q_market.price;
                ismarket = true;
            } else {
                entry = entrytext.parse::<Decimal>()?;
            }

            if (q_account.leverage * risk) > dec!(100) {
                bail!("You are at risk of liquidation so the trade cannot take place. Check leverage and risk.");
            }

            let values = super::misc::OrderCalcEntry {
                totalliquid: totalliquid,
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
            println!();
            println!("    Order Size: {} {}", calculation.quantity.round_dp(6), &quote_currency);
            println!("    SL-TP Ratio : {}R", calculation.tpslratio.round_dp(2));
            println!("    % Of ({}) {} Liquidity: {}", subaccount, &quote_currency, ((calculation.quantity / (totalliquid * if isfuture {q_account.leverage} else {dec!(1)})) * dec!(100)).round_dp(2));

            let fees;
            match ismarket {
                true => {
                    fees = (calculation.quantity * account.taker_fee).round_dp(3);
                },
                false => {
                    fees = (calculation.quantity * account.maker_fee).round_dp(3);
                }
            };

            let feesofsub = fees/totalliquid;

            println!("    Fees: {} {} ({}% of sub)",  fees, &quote_currency, feesofsub.round_dp(4));
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

            let q_main_order;

            match ismarket {
                true => {
                    //change this to normal market order
                    q_main_order = api.place_order(
                        pair,
                        if calculation.islong {ftx::rest::Side::Buy} else {ftx::rest::Side::Sell},
                        None,
                        ftx::rest::OrderType::Market, //change this later so you dont get terrible fees
                        calculation.quantity/q_market.price,
                        None,
                        None,
                        None, //when implimenting close limit order change to true
                        None
                    ).await?;
                },
                false => {
                    if entry > q_market.price {
                        //trigger stop
                        q_main_order = api.place_trigger_order(
                            pair,
                            if calculation.islong {ftx::rest::Side::Buy} else {ftx::rest::Side::Sell},
                            calculation.quantity/q_market.price,
                            ftx::rest::OrderType::Stop,
                            entry,
                            None,
                            None,
                            None,
                            None
                        ).await?;
                    } else {
                        //limit
                        q_main_order = api.place_order(
                            pair,
                            if calculation.islong {ftx::rest::Side::Buy} else {ftx::rest::Side::Sell},
                            None,
                            ftx::rest::OrderType::Limit,
                            calculation.quantity/q_market.price,
                            None,
                            None,
                            None, //when implimenting close limit order change to true
                            None
                        ).await?;
                    }
                }
            }

            println!("{:#?}", q_main_order);

            //take profit and stop loss
        },
        "stop" => {
            //TBF
            //stops order
        },
        "lev" => {
            let q_account = api.get_account().await?;
            println!("  Current Leverage: {}", q_account.leverage);
        },
        "subs" => {
            let q_subaccounts = api.get_subaccounts().await?;

            let mut subcounter:i32 = 0;
            for subacc in &q_subaccounts {
                subcounter += 1;
                println!("{}. {}", subcounter, boldt(&subacc.nickname));
            }
        },
        //function to change the current pair
        "pair" => {
            let temppair:String;
            //take input for auto mode
            //accept user input in two parts
            println!("  Change pair:");
            let prefix = ask("[Prefix]")?;
            let suffix = ask("[Suffix]")?;

            //format parts into temppair
            temppair = formattedpair([prefix.as_str(), suffix.as_str()]);
               

            let q_markets = api.get_markets().await?;
            let mut isrealpair:bool = false;
        
            for item in &q_markets {
                if item.name == temppair.as_str() {
                    isrealpair = true;
                }
            }
                    
            match isrealpair {
                true => {
                    println!("    {}", boldt("Success (pair found)"));

                    //gets price of pair
                    let q_market = api.get_market(&temppair.as_str()).await?;

                    //changes pair value to new chosen pair
                    *pair = temppair;
                    println!("    Price ({}): {}{}", pair, q_market.price, getsuffixsymbol(pair.as_str()));
                },
                false => {
                    println!("    {}", boldt("Error (pair not found)"));
                }
            }
        },
        //function to return the price of the current pair
        "p"|"price" => {     
            let q_market = api.get_market(&pair.as_str()).await?;
            println!("  Mid: {}", q_market.price);
            println!("  Ask: {}", q_market.ask);
            println!("  Bid: {}", q_market.bid);
        },
        //function to get balance of current subaccount
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
        //display orderbook of current pair
        "ob" | "orderbook" => {
            println!("{}", boldt("Orderbook"))
        },
        //get list of all markets (including futures)
        "allmarkets" => {
            let q_markets = api.get_markets().await?;
            for item in &q_markets {
                print!("{} | ", item.name)
            }
        },
        //get list of all futures
        "allfutures" => {
            let q_futures = api.get_futures().await?;
            for item in &q_futures {
                print!("{} | ", item.name)
            }
        },
        //get list of all markets (including futures) and prices
        "listprices" => {
            let q_markets = api.get_markets().await?;
            for item in &q_markets {
                println!("{} - {}", item.name, item.price)
            }
        }
        //get account object
        "account" => {
            *account = api.get_account().await?;
            println!("{:#?}", account);
        },
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