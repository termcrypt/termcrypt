use ftx::rest::Rest;
use ftx::rest::Error as errr;

use super::utils::{
    retout as ret,
    askout as ask,
    boldt as boldt,
    formattedpair,
    getsuffixsymbol
};

pub struct FtxHcStruct {
    pub pair: String,
    pub subaccount: String
}

pub async fn handle_commands<'a>(x:&str, subaccount:&mut String, pair:&mut String, api:&mut Rest) -> Result<FtxHcStruct, errr> {
    //handles the command given by the user
    match x {
        //function to make sure user does not give wrong input
        "sub"|"search" => {
            println!("  !For this function, please use this format: {} [parameters]", x);
        },
        //clears terminal
        "clear" => print!("{}[2J", 27 as char),
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
                    *subaccount = "def".to_string();
                    println!("    {}", boldt("Returning to default account"));
                },
                _ => {
                    let allsubaccounts = api.get_subaccounts().await?;

                    let mut didfind = false;
                    for subacc in &allsubaccounts{
                        if subacc.nickname.as_str() == subtosearch {
                            *subaccount = subtosearch.to_string();
                            didfind = true;
                            println!("    {}", boldt("Success (found subaccount)"));
                        }
                    }
                    if !didfind {
                        println!("  No subaccount found called - {}", subtosearch);
                    }
                }
            }
        },
        "subs" => {
            //TBF
        },
        //function to change the current pair
        "pair" => {
            let temppair:String;
            //take input for auto mode
            //accept user input in two parts
            ret("Change pair:");
            let prefix = ask("[Prefix]");
            let suffix = ask("[Suffix]");

            //format parts into temppair
            temppair = formattedpair([prefix.as_str(), suffix.as_str()]);
               

            let query = api.get_markets().await?;
            let mut isrealpair:bool = false;
        
            for item in &query {
                if item.name == temppair.as_str() {
                    isrealpair = true;
                }
            }
                    
            match isrealpair {
                true => {
                    println!("    {}", boldt("Success (pair found)"));

                    //gets price of pair
                    let query = api.get_market(&temppair.as_str()).await?;

                    //changes pair value to new chosen pair
                    *pair = temppair;
                    println!("    Price ({}): {}{}", pair, query.price, getsuffixsymbol(pair.as_str()));
                },
                false => {
                    println!("    {}", boldt("Error (pair not found)"));
                }
            }
        },
        x if x.starts_with("search ") => {
            //grabs second part of command to search for
            let tosearch:String = x.split("search ").collect();
            let markets = api.get_markets().await?;

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
            println!("  {} {}", matched_count, boldt("Pairs Found"))
        },
        //function to return the price of the current pair
        "price"|"p" => {     
            let query = api.get_market(&pair.as_str()).await?;
            println!("  {}: {}", pair, query.price);
        },
        //function to get balance of current subaccount
        "bal"|"balance" => {
            match subaccount.as_str() {
                //default account (no subaccount chosen)
                "def" => {
                    let balances = api.get_wallet_balances().await?;
                    println!("[{} Balance types]", balances.len());
                    for balance in &balances {
                        println!("  {}", boldt(&balance.coin));
                        println!("     Free:  {}", &balance.free);
                        println!("     Total: {}", &balance.total);
                        println!();
                    }
                },
                //other cases (subaccount chosen)
                _ => {
                    let balances = api.get_subaccount_balances(subaccount).await?;
                    println!("[{} Balance types]", balances.len());
                    for balance in balances {
                        println!("  {}", boldt(&balance.coin));
                        println!("     Free:  {}", &balance.free);
                        println!("     Total: {}", &balance.total);
                        println!();
                    }
                }
            }
            //TBF
        },
        //get list of all markets (including futures)
        "allmarkets" => {
            let query = api.get_markets().await?;
            for item in &query {
                print!("{} | ", item.name)
            }
        },
        //get list of all futures
        "allfutures" => {
            let query = api.get_futures().await?;
            for item in &query {
                print!("{} | ", item.name)
            }
        },
        //get list of all markets (including futures) and prices
        "listprices" => {
            let query2 = api.get_markets().await?;
            for item in &query2 {
                println!("{} - {}", item.name, item.price)
            }
        }
        //get account object
        "account" => {
            println!("{:#?}", api.get_account().await?);
        },
        "rawmarkets" => {
            let query = api.get_markets().await?;
            println!("{:#?}", query);
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