use crate::bybit_core::bybit_ws::*;
use anyhow::{Error as AnyHowError, Result};

//use crate::utils::termbug;

pub async fn websocket_init(
	bybit_pub_key: String,
	bybit_priv_key: String, /* Change to struct later */
) -> Result<(), AnyHowError> {
	tokio::spawn(async {
		match bybit_websocket(bybit_pub_key, bybit_priv_key).await {
			Ok(_x) => { /*:L*/ }
			Err(_e) => {
				todo!("Websocket init failed: {}", _e);
				//termbug::error("FATAL Bybit websocket error (Opening trades is unsafe) {_e:?}");
			}
		}
	});
	Ok(())
}
