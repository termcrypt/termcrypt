use anyhow::{bail, Result};

pub fn bybit_get_base(mut fp: String) -> Result<String> {
    //fp is full pair
    fp = fp.to_uppercase();
    let base: String;
    match fp {
        fp if fp.ends_with("USDT") => {
            base = "USDT".to_string()
        },
        fp if fp.ends_with("BTC") => {
            base = "BTC".to_string()
        },
        fp if fp.ends_with("USDC") => {
            base = "USDC".to_string()
        },
        fp if fp.ends_with("USD") => {
            base = fp.strip_suffix("USD").unwrap().to_string();
        },
        fp if fp.ends_with("USD0325") => {
            base = fp.strip_suffix("USD0325").unwrap().to_string();
        },
        fp if fp.ends_with("USD0624") => {
            base = fp.strip_suffix("USD0624").unwrap().to_string();
        }
        _ => {
            bail!("Could not find base currency for pair.");
        }
    }
    Ok(base)
}