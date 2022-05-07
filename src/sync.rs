use crate::bybit_core::bybit_sync;
use anyhow::{Error, Result};

// Function to syncronize what was missed when termcrypt was not running
pub async fn _startup_sync(bybit_api: Option<&mut bybit::http::Client>) -> Result<(), Error> {
	bybit_sync::startup_sync(bybit_api.unwrap()).await?;

	Ok(())
}
