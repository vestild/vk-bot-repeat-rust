#[macro_use]
extern crate lazy_static;
extern crate log;

extern crate simplelog;
use simplelog::*;
use log::{info, trace, warn};

mod config;

fn main() {
    SimpleLogger::init(LevelFilter::Trace, Config::default()).unwrap();

    info!(target: "main", "token {:?}", format_secret(config::CONF.token()))   ;
}

fn format_secret(s: &str) -> String {
    let mut r = String::from(s);
    let len = s.len();
    if len < 3 {
        return String::from("**");
    } else if len <= 6 {
        r.replace_range(1.., &"*".repeat(len - 1))
    } else {
        r.replace_range(2..len - 2, &"*".repeat(len - 4))
    }
    r
}

#[test]
fn test_log_secret() {
    assert_eq!(format_secret(""), "**");
    assert_eq!(format_secret("1"), "**");
    assert_eq!(format_secret("123"), "1**");
    assert_eq!(format_secret("123456"), "1*****");
    assert_eq!(format_secret("1234567"), "12***67");
    assert_eq!(format_secret("123456789"), "12*****89");
}