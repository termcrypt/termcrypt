use anyhow::{
	Error,
	//bail
	Result,
};

use super::utils::{askout as ask, boldt};

pub fn data_location() -> String {
	format!("{}/termcrypt/db/", dirs::data_dir().unwrap().display())
}

pub fn get_db_info() -> Result<super::Config, Error> {
	//open database
	let db: sled::Db = sled::open(data_location().as_str())?;
	//println!("{:#?}", db);

	//set data point variables to specified db / default values
	let default_pair = get_dbinf_by_entry(&db, "default_pair", Some("BTC-PERP"), None)?;
	let default_sub = get_dbinf_by_entry(&db, "default_sub", Some("def"), None)?;
	//
	let ftx_pub_key = get_dbinf_by_entry(&db, "ftx_pub_key", None, Some("public FTX API key"))?;
	let ftx_priv_key =
		get_dbinf_by_entry(&db, "ftx_priv_key", None, Some("private FTX API secret"))?;
	//
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
) -> Result<String, Error> {
	let value = match db.get(key_name)? {
		Some(val) => String::from_utf8(val.to_vec())
			.expect("Something is wrong with your database. Please open an issue on github."),
		None => {
			if let Some(default) = default_value {
				default.to_string()
			} else {
				print!("{}[2J", 27 as char);
				println!(
					"{}",
					boldt("termcrypt needs configuration for first time use.")
				);
				println!();
				let input = ask(&format!("Please enter your {}", name.unwrap()))?;
				insert_db_info_entry(db, key_name, &input)?
			}
		}
	};
	Ok(value)
}

pub fn insert_db_info_entry(db: &sled::Db, key_name: &str, value: &str) -> Result<String, Error> {
	db.insert(key_name, value)?;
	Ok(value.to_string())
}
