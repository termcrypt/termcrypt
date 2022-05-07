pub fn _formatted_pair(pairr: [&str; 2]) -> String {
	let divider = match pairr[1].to_uppercase().as_str() {
		"PERP" | "1231" => "-",
		_ => "/",
	};
	[
		pairr[0].to_uppercase(),
		divider.to_string(),
		pairr[1].to_uppercase(),
	]
	.concat()
}

pub fn _get_suffix_symbol(pair: &str) -> &str {
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
