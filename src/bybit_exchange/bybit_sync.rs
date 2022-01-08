use anyhow::{Error as AnyHowError, Result};
use bybit::{http};

pub async fn bybit_startup_sync(_api: &mut http::Client) -> Result<(), AnyHowError> {
    //tbf
    Ok(())
}