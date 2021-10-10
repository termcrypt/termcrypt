use sled;

use anyhow::{
    Result,
    Error,
    //bail
};

use super::utils::{
    askout as ask,
    boldt
};

pub fn get_db_info() -> Result<super::Config, Error> {
    //open database
    let db:sled::Db = sled::open("db"/*path*/)?;
    //println!("{:#?}", db);
    
    //set data point variables to specified db / default values
    let pair = get_dbinf_by_entry(&db, "pair", Some("BTC-PERP"), None)?;
    let ftx_pub_key = get_dbinf_by_entry(&db, "ftx_pub_key", None, Some("public FTX API key"))?;
    let ftx_priv_key = get_dbinf_by_entry(&db, "ftx_priv_key", None, Some("private FTX API secret"))?;

    return Ok(super::Config {
        pair: pair,
        ftx_pub_key: ftx_pub_key,
        ftx_priv_key: ftx_priv_key
    })
}

pub fn get_dbinf_by_entry(db:&sled::Db, key_name:&str, default_value:Option<&str>, name:Option<&str>) -> Result<String, Error> {
    let value = match db.get(key_name)? {
        Some(val) => String::from_utf8(val.to_vec()).expect("Something is wrong with your database. Please open an issue on github."),
        None => if let Some(default) = default_value {
            default.to_string()
        } else {
            print!("{}[2J", 27 as char);
            println!("{}", boldt("termcrypt needs configuration for first time use."));
            println!();
            let input = ask(&format!("Please enter your {}", name.unwrap()))?;
            insert_db_info_entry(db, key_name, &input)?
        }
    };
    Ok(value)
}

pub fn insert_db_info_entry(db:&sled::Db, key_name:&str, value:&str) -> Result<String, Error> {
    db.insert(key_name, value)?;
    return Ok(value.to_string());
}