#[macro_use(scanln)] extern crate scanln;
mod ftx_inter;
mod misc;
mod utils;
use dotenv::dotenv;

use ftx::{
    options::Options,
    rest::{Rest},
};

/* global variable
static VAR:&str = "string";
*/

#[tokio::main]
async fn main() {
    //initiates dotenv variables
    dotenv().ok();

    //creates subbaccount and pair defaults
    let mut subaccount: String = "def".to_string();
    let mut pair: String = "BTC/USD".to_string();

    //uses .env to initiate api environment
    let mut api = Rest::new(Options::from_env());

    //outputs version and ascii art
    utils::version();

    loop {
        /*start of loop
        takes input from user through terminal-like interface*/
        let input = scanln!("[{}]({})> ", subaccount.as_str(), pair.as_str());
        let strinput = input.as_str();

        //ftx command handling
        match ftx_inter::handle_commands(strinput, &mut subaccount, &mut pair, &mut api).await {
            //error handling
            Ok(_x) => {
                //subaccount = x.subaccount;
                //pair = x.pair;
            },
            Err(e) => {
                println!("  !You did something stupid: {:?}", e);
                println!();
                continue;
            }
        }

        //miscellaneous command handling
        misc::handle_commands(strinput);

        //adds padding to previous output
        println!();
    }
}