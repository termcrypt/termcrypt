use chrono::{
    NaiveTime,
    Utc,
    DateTime,
    TimeZone,
    Duration,
};

use anyhow::{
    Result,
    Error,
    bail,
};

use super::utils::{
    boldt,
    //round_dp_up,
    round_dp_tz,
    wideversion,
    slimversion,
};

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

//use super::utils::boldt as boldt;

//Command Handling
pub fn handle_commands(x:&str, wide:bool, loop_iteration:i32) {
    match x {
        "clr"|"clear" => print!("{}[2J", 27 as char),
        "ses"|"sessions" => {
            //trading sessions time information
            println!("{}", boldt("Trading Sessions"));
            let utc_time = Utc::now().time();
            let utc_now = Utc::now();

            let mut circlecolor:&str;   
            let mut timetoevent:String;
            let mut eventtime;
            let mut isopen:bool;

            pub struct Times {
                hourstoevent: Decimal,
                minutestoevent: Decimal,
                _secondstoevent: Decimal
            }

            fn times_until<Tz2: TimeZone>(eventtiming:DateTime<Tz2>) -> Times {
                let duration = eventtiming.signed_duration_since(Utc::now()).to_std().unwrap();

                let hours = (Decimal::from_str(duration.as_secs().to_string().as_str()).unwrap() / dec!(60)) / dec!(60);
                let minutes = (hours - round_dp_tz(hours, 0)) * dec!(60);
                let seconds = (minutes - round_dp_tz(minutes, 0)) * dec!(60);

                Times {
                    hourstoevent: hours,
                    minutestoevent: minutes,
                    _secondstoevent: seconds,
                }
            }

            //NY SESSION

            if utc_time >= NaiveTime::from_hms(12,0,0) && utc_time < NaiveTime::from_hms(18,0,0){
                circlecolor = "🟢";
                eventtime = utc_now.date().and_hms(18,0,0);
                isopen = true;    
            } else {
                circlecolor = "🔴";
                isopen = false;

                if utc_time < NaiveTime::from_hms(12,0,0) && utc_time >= NaiveTime::from_hms(0,0,0) {
                    eventtime = utc_now.date().and_hms(12,00,0);
                } else {
                    eventtime = (utc_now + Duration::days(1)).date().and_hms(12,00,0);
                }
            }

            let times = times_until(eventtime);
            timetoevent = format!("{} in: {}h {}m", if isopen {"Closes"} else {"Opens"}, round_dp_tz(times.hourstoevent, 0), round_dp_tz(times.minutestoevent, 0)/*, round_dp_tz(secondstoevent, 0)*/);

            println!("  {} NY (OPT)", circlecolor);
            println!("    {}", timetoevent);
            println!("    (12AM-8PM UTC)");
            println!();

            //ASIA SESSION

            if utc_time >= chrono::NaiveTime::from_hms(23,0,0) || utc_time < chrono::NaiveTime::from_hms(4,0,0) {
                circlecolor = "🟢";
                isopen = true;

                if utc_time >= chrono::NaiveTime::from_hms(23,0,0) && utc_time <= chrono::NaiveTime::from_hms(23,59,0) {
                    eventtime = (utc_now + Duration::days(1)).date().and_hms(4,0,0);
                } else {
                    eventtime = utc_now.date().and_hms(4,0,0);
                }
            } else {
                circlecolor = "🔴";
                eventtime = utc_now.date().and_hms(23,0,0);
                isopen = false;
            }

            let times = times_until(eventtime);
            timetoevent = format!("{} in: {}h {}m", if isopen {"Closes"} else {"Opens"}, round_dp_tz(times.hourstoevent, 0), round_dp_tz(times.minutestoevent, 0)/*, round_dp_tz(secondstoevent, 0)*/);

            println!("  {} ASIA (OPT)", circlecolor);
            println!("    {}", timetoevent);
            println!("    (11PM-4AM UTC)"); 
        },
        "date" => {
            println!("  {}", chrono::Utc::now().format("[UTC] %b %-d %Y"));
            println!("  {}", chrono::Local::now().format("[Now] %b %-d %Y"));
        },
        "time" => {
            println!("  {}", chrono::Utc::now().format("[UTC] %H:%M:%S"));
            println!("  {}", chrono::Local::now().format("[Now] %H:%M:%S"));
        },
        "afetch" => {
            if wide {
                wideversion();
            } else {
                slimversion();
            };
        },
        "ping" => {
            println!("{}", boldt("🏓 Pong!"));
        },
        "count" => {
            println!("{}", loop_iteration);
        },
        "q" => {
            println!("Exiting...");
            println!();
            println!("{}", boldt("Thank you for using termcrypt ;)"));
            println!();
            panic!();
        }
        _=> ()
    }
}


//Calculations
pub struct OrderCalcEntry {
    pub total_liquid: Decimal,
    pub risk: Decimal,
    pub stoploss: Decimal,
    pub takeprofit: Decimal,
    pub entry: Decimal,
}

#[derive(Debug)]
pub struct OrderCalcExit {
    pub quantity: Decimal,
    pub islong: bool,
    pub tpslratio: Decimal,
}

pub fn calculate_order(ov:OrderCalcEntry) -> Result<OrderCalcExit, Error> {
    let mut islong = true;
    let mut quantity:Decimal = dec!(1);
    let mut tpslratio:Decimal = dec!(1.0);

    //Checks if parameters are correct
    //More should be added here for more cases

    
    if !(
        ov.risk <= dec!(100) &&
        ov.stoploss != ov.takeprofit &&
        ov.stoploss != ov.entry &&
        ov.takeprofit != ov.entry &&
        ov.risk >= dec!(0.1)
    ) {
        bail!("Invalid parameters when calculating order size");
    }
    

    //Checks for Long
    if
        ov.stoploss < ov.takeprofit &&
        ov.entry > ov.stoploss &&
        ov.entry < ov.takeprofit
    {
        islong = true;
        quantity = ov.total_liquid * (ov.risk / ((dec!(1) - (ov.stoploss / ov.entry)) * dec!(100)));
        tpslratio = (ov.takeprofit - ov.entry) / (ov.entry - ov.stoploss);  //toFixed(2)
    } 
    
    //Cheks for Short
    else if
        ov.stoploss > ov.takeprofit &&
        ov.entry < ov.stoploss &&
        ov.entry > ov.takeprofit
    {
        islong = false;
        quantity = ov.total_liquid * (ov.risk / ((dec!(1) - (ov.entry / ov.stoploss)) * dec!(100)));
        tpslratio = (ov.entry - ov.takeprofit) / (ov.stoploss - ov.entry);  //toFixed(2)
    }

    Ok(
        OrderCalcExit {
            quantity,
            islong,
            tpslratio,
        }
    )

}
