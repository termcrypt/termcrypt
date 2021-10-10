mod ftx_inter;
mod misc;
mod utils;
mod advanced_orders;
mod db;

use dotenv::dotenv;
//use std::thread;
use rustyline::error::ReadlineError;
use rustyline::Editor;

use ftx::{
    options::Options,
    rest::Rest
};

use db::{
    get_db_info
};

use terminal_size::{Width, Height, terminal_size};
static VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub struct Config {
    pub pair: String,
    pub ftx_pub_key: String,
    pub ftx_priv_key: String
}

#[tokio::main]
async fn main() { 
    //initiates database
    let mut db_info = get_db_info().unwrap();

    let mut pair:String = db_info.pair;
    
    //db.insert("yo", "bruh").unwrap();

    //creates subbaccount and pair defaults
    let mut subaccount: String = "def".to_string();
    //let mut pair: String = "BTC/USD".to_string();

    //uses .env to initiate api environment
    dotenv().ok();
    let mut api = Rest::new(Options {
        key: Some(db_info.ftx_pub_key),
        secret: Some(db_info.ftx_priv_key),
        subaccount: None,
        endpoint: ftx::options::Endpoint::Com
    });

    //gets user account
    let mut q_account = api.get_account().await.unwrap();

    //gets terminal size
    let size = terminal_size();
    let mut wide = true;

    if let Some((Width(w), Height(_h))) = size {
        if w<70 {
            wide = false;
        }
    } else {
        wide = false
    }

    //outputs version and ascii art
    if wide {
        utils::wideversion();
    } else {
        utils::slimversion();
    };
    println!();

    let mut rl = Editor::<()>::new();
    let mut loop_iteration:i32 = 0;

    loop {
        //INITIATE DB
        
        db_info = get_db_info().unwrap();

        //Start of loop
        //Takes input from user through terminal-like interface*/
        let readline = rl.readline(format!("[{}]({})> ", subaccount.as_str(), pair.as_str()).as_str());

        match readline {
            Ok(readline) => {
                rl.add_history_entry(readline.as_str());
                //ftx command handling
                match ftx_inter::handle_commands(
                    //make this a struct one day lazy ass
                    readline.as_str(),
                    &mut subaccount,
                    &mut pair,
                    &mut api,
                    &mut q_account,
                    wide
                ).await {
                    Ok(_x) => {
                        //subaccount = x.subaccount;
                        //pair = x.pair;
                    },
                    Err(e) => {
                        println!();
                        eprintln!("!! Function Exited: {:?} !!", e);
                        println!();
                        continue;
                    }
                }
                //miscellaneous command handling
                misc::handle_commands(
                    //make this a struct one day lazy ass
                    readline.as_str(),
                    wide,
                    loop_iteration
                );

                //adds padding
                println!();
            },
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("Exiting...");
                println!();
                println!("{}", utils::boldt("Thank you for using termcrypt ;)"));
                break
            },
            Err(e) => {
                println!();
                eprintln!("!! Something bad happened, be scared: {:?} !!", e);
                println!();
                break;
            }
        }
        loop_iteration += 1;
    }
}