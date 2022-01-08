use super::bybit_exchange::{bybit_sync::*};
use anyhow::{Error as AnyHowError, Result};

//function to synchronize what was missed when termcrypt was not running
pub async fn startup_sync(bybit_api: Option<&mut bybit::http::Client>/* when multi exchanges are added, pass enabled exchanges here */) -> Result<(), AnyHowError> {
    bybit_startup_sync(bybit_api.unwrap()).await?;

    Ok(())
}