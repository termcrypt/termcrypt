use anyhow::{Error, Result};
use bybit::http;

pub async fn startup_sync(_api: &mut http::Client) -> Result<(), Error> {
	//tbf
	Ok(())
}
