pub fn ftx_formattedpair(pairr: [&str; 2]) -> String {
	let divider: &str;
	match pairr[1].to_uppercase().as_str() {
		"PERP" | "1231" => divider = "-",
		_ => divider = "/",
	}
	[
		pairr[0].to_uppercase(),
		divider.to_string(),
		pairr[1].to_uppercase(),
	]
	.concat()
}

pub fn ftx_getsuffixsymbol(pair: &str) -> &str {
	let usdsigns: [&str; 3] = ["USD", "PERP", "USDT"];
	for item in &usdsigns {
		if pair.ends_with(item) {
			return "$";
		}
	}
	match pair {
		x if x.ends_with("BTC") => "₿",
		x if x.ends_with("ETH") => "₿",
		_ => "",
	}
}
