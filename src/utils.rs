use ansi_term::Style;
use ansi_term::ANSIGenericString;
use core::fmt::Display;

pub fn retout(x: impl Display) {
    //retout handles and adds formatting to the responses from the modules' command handling.
    println!("  {}", x);
}

pub fn askout(prefix:&str) -> String {
    return scanln!("  {}>> ", prefix);
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

pub fn version() {
    print!("{}[2J", 27 as char);
    println!();
    println!("  _______ _______ ______ _______ ______ ______ ___ ___ ______ _______ ");
    println!(" |_     _|    ___|   __ ⑊   |   |      |   __ ⑊   |   |   __ ⑊_     _|");
    println!("   |   | |    ___|      <       |   ---|      <⑊     /|    __/ |   |  ");
    println!("   |___| |_______|___|__|__|_|__|______|___|__| |___| |___|    |___|  ");
    println!();
    println!("  v0.1.1. License: AGPL3+");
    println!();
}