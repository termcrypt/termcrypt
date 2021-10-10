use ansi_term::Style;
use ansi_term::ANSIGenericString;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use anyhow::{
    Error,
    bail
};
use rust_decimal::{
    Decimal,
    RoundingStrategy
};

pub fn askout(prefix:&str) -> Result<String, Error> {
    let mut rl = Editor::<()>::new();
    let readline = rl.readline(format!("  {}>> ", prefix).as_str());
    
    match readline {
        //add some smart error handling to re-loop from askout function
        Ok(readline) => {
            if readline == "qq".to_string() {
                bail!("User stopped");
            } else {
                return Ok(readline.to_string())
            }
        },
        Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
            println!("Exiting...");
            println!();
            println!("{}", boldt("Thank you for using termcrypt ;)"));
            println!();
            panic!();
        }
        Err(e) => {
            panic!("{}", e);
        }
    }
}

pub fn boldt(text:&str) -> ANSIGenericString<'_, str>{
    return Style::new().bold().paint(text);
}

pub fn formattedpair(pairr:[&str;2]) -> String {
    let divider:&str;
    match pairr[1].to_uppercase().as_str(){
        "PERP"|"1231" => {
            divider = "-"
        },
        _ => {
            divider = "/"
        }
    }
    return [pairr[0].to_uppercase(), divider.to_string(), pairr[1].to_uppercase()].concat();
}

pub fn getsuffixsymbol(pair:&str) -> &str {
    let usdsigns:[&str; 3] = ["USD", "PERP", "USDT"];
    for item in &usdsigns {
        if pair.ends_with(item) {
            return "$";
        }
    }

    if pair.ends_with("BTC") {
        return "₿";
    }
    if pair.ends_with("ETH") {
        return "₿";
    }
    return "";
}

pub fn _round_dp_up(num:Decimal, places:u32) -> Decimal {
    return num.round_dp_with_strategy(places, RoundingStrategy::MidpointAwayFromZero);
}

pub fn round_dp_tz(num:Decimal, places:u32) -> Decimal {
    return num.round_dp_with_strategy(places, RoundingStrategy::ToZero);
}

pub fn _sideret(text:&str) {
    println!();
    println!("{}", boldt(text));
    println!("Continue your previous location ⌄ below ⌄");
}

pub fn wideversion() {
    print!("{}[2J", 27 as char);
    println!();
    println!("  _______ _______ ______ _______ ______ ______ ___ ___ ______ _______ ");
    println!(" |_     _|    ___|   __ ⑊   |   |      |   __ ⑊   |   |   __ ⑊_     _|");
    println!("   |   | |    ___|      <       |   ---|      <⑊     /|    __/ |   |  ");
    println!("   |___| |_______|___|__|__|_|__|______|___|__| |___| |___|    |___|  ");
    println!();
    println!("  v{}. License: 🟢 AGPL3+", super::VERSION);
}

pub fn slimversion() {
    print!("{}[2J", 27 as char);
    println!();
    println!("  {}", boldt("<termcrypt>"));
    println!();
    println!("  v{}. License: 🟢 AGPL3+", super::VERSION);
}