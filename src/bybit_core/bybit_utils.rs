use anyhow::{bail, Result};

pub fn bybit_get_base(mut fp: String) -> Result<String> {
	//fp is full pair
	fp = fp.to_uppercase();
	let base = match fp {
		fp if fp.ends_with("USDT") => "USDT".to_string(),
		fp if fp.ends_with("BTC") => "BTC".to_string(),
		fp if fp.ends_with("USDC") => "USDC".to_string(),
		fp if fp.ends_with("USD") => {
			fp.strip_suffix("USD").unwrap().to_string()
		}
		fp if fp.ends_with("USD0325") => {
			fp.strip_suffix("USD0325").unwrap().to_string()
		}
		fp if fp.ends_with("USD0624") => {
			fp.strip_suffix("USD0624").unwrap().to_string()
		}
		_ => {
			bail!("Could not find base currency for pair.");
		}
	};
	Ok(base)
}
