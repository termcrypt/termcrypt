use anyhow::{
	Error,
	//bail
	Result,
};

use ftx::{options::Options, rest::*};

use super::utils::{askout as ask, boldt};

pub fn database_location() -> String {
	format!("{}/termcrypt/db/", dirs::data_dir().unwrap().display())
}

pub fn history_location() -> String {
	format!("{}/termcrypt/history/", dirs::data_dir().unwrap().display())
}

pub async fn get_db_info() -> Result<super::Config, Error> {
	//open database
	let db: sled::Db = sled::open(database_location().as_str())?;
	//println!("{:#?}", db);

	//set data point variables to specified db / default values
	let default_pair = get_dbinf_by_entry(&db, "default_pair", Some("BTC-PERP"), None, false)?;
	let default_sub = get_dbinf_by_entry(&db, "default_sub", Some("def"), None, false)?;
	let mut ftx_pub_key;
	let mut ftx_priv_key;

	let mut force_retype = false;
	loop {
		ftx_pub_key = get_dbinf_by_entry(
			&db,
			"ftx_pub_key",
			None,
			Some("public FTX API key"),
			force_retype,
		)?;
		ftx_priv_key = get_dbinf_by_entry(
			&db,
			"ftx_priv_key",
			None,
			Some("private FTX API secret"),
			force_retype,
		)?;

		let api = Rest::new(Options {
			key: Some(ftx_pub_key.to_string()),
			secret: Some(ftx_priv_key.to_string()),
			subaccount: None,
			endpoint: ftx::options::Endpoint::Com,
		});

		match api.request(GetAccount).await {
			Ok(_x) => break,
			Err(e) => {
				println!();
				println!("{}", boldt(format!("{}", e).as_str()));
				println!(
					"  {}",
					boldt("!! API keys are not valid, please try again !!")
				);
				force_retype = true;
				continue;
			}
		}
	}
	Ok(super::Config {
		default_pair,
		default_sub,
		ftx_pub_key,
		ftx_priv_key,
	})
}

pub fn get_dbinf_by_entry(
	db: &sled::Db,
	key_name: &str,
	default_value: Option<&str>,
	name: Option<&str>,
	force_retype: bool,
) -> Result<String, Error> {
	let value = if !force_retype {
		match db.get(key_name)? {
			Some(val) => String::from_utf8(val.to_vec())
				.expect("Something is wrong with your database. Please open an issue on github."),
			None => {
				if let Some(default) = default_value {
					default.to_string()
				} else {
					//print!("{}[2J", 27 as char);
					println!();
					println!(
						"{}",
						boldt("termcrypt needs configuration for first time use.")
					);
					println!();
					let input = ask(&format!("Please enter your {}", name.unwrap()), None)?;
					insert_db_info_entry(db, key_name, &input)?
				}
			}
		}
	} else {
		let input = ask(&format!("Please enter your {}", name.unwrap()), None)?;
		insert_db_info_entry(db, key_name, &input)?
	};
	Ok(value)
}

pub fn insert_db_info_entry(db: &sled::Db, key_name: &str, value: &str) -> Result<String, Error> {
	db.insert(key_name, value)?;
	Ok(value.to_string())
}
