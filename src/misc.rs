use chrono;

use super::utils::retout as ret;
//use super::utils::boldt as boldt;

pub fn handle_commands(x:&str) {
    match x {
        "date" => {
            ret(chrono::Local::now().format("[Now] %b %-d %Y"));
            ret(chrono::Utc::now().format("[UTC] %b %-d %Y"));
        },
        "time" => {
            ret(chrono::Local::now().format("[Now] %H:%M:%S"));
            ret(chrono::Utc::now().format("[UTC] %H:%M:%S"));
        },
        _=> ()
    }
}