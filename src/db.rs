use sled;
use sled::{
    IVec
};

use anyhow::{
    Result,
    Error,
    //bail
};

use super::utils::{
    askout as ask
};

pub fn get_db_info() -> Result<super::Config, Error> {
    //open database
    let db:sled::Db = sled::open("db"/*path*/)?;
    //let defaultpair = db.get("defaultpair").unwrap().flat_map(|b| String::from_utf8(b)).unwrap_or("BTC-PERP").unwrap();
    let pair = get_cfg_by_entry(db, "pair", "BTC-PERP", false)?;

    return Ok(super::Config {
        pair: pair
    })
}


pub fn get_cfg_by_entry(db:sled::Db, keyname:&str, defaultvalue:&str, doinsert:bool) -> Result<String, Error> {
    //returns utf8 as string from database. Inserts default if not found.
    return Ok(db.get(keyname)?.and_then(|iveccy| String::from_utf8(iveccy.to_vec()).ok()).unwrap_or(if doinsert {insert_config_default(db, keyname, defaultvalue)?} else {defaultvalue.to_string()}));
}


pub fn insert_config_default(db:sled::Db, keyname:&str, value:&str) -> Result<String, Error> {
    db.insert(keyname, value)?;
    return Ok(value.to_string());
}