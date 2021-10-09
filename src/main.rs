mod ftx_inter;
mod misc;
mod utils;
mod advanced_orders;
use dotenv::dotenv;
//use std::thread;
use rustyline::error::ReadlineError;
use rustyline::Editor;

use ftx::{
    options::Options,
    rest::Rest
};

use terminal_size::{Width, Height, terminal_size};

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

    let mut q_account = api.get_account().await.unwrap();

    //outputs version and ascii art
    if wide {
        utils::wideversion();
    } else {
        utils::slimversion();
    };
    println!();

    let mut rl = Editor::<()>::new();

    loop {
        /*start of loop
        takes input from user through terminal-like interface*/

        //let input = scanln!("[{}]({})> ", subaccount.as_str(), pair.as_str());
        //let strinput = input.as_str();

        let readline = rl.readline(format!("[{}]({})> ", subaccount.as_str(), pair.as_str()).as_str());
        match readline {
            Ok(readline) => {
                rl.add_history_entry(readline.as_str());
                //ftx command handling
                match ftx_inter::handle_commands(readline.as_str(), &mut subaccount, &mut pair, &mut api, &mut q_account, wide).await {
                    //error handling
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
                misc::handle_commands(readline.as_str(), wide);

                //adds padding to previous output
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
    }
}