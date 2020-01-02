#![recursion_limit="1024"]
#[macro_use]

extern crate lazy_static;

use simplelog::*;
use log::{info};
use tokio::run;
use futures::future::lazy;

mod config;
mod init;
mod server_config;

fn main() {
    SimpleLogger::init(LevelFilter::Debug, Config::default()).unwrap();

    info!(target: "main", "token {:?}", format_secret(config::CONF.token()));
    let sc = server_config::ConfigProvider::new();

    run(lazy(|| {
        init::run(sc)
    }));
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

#[cfg(test)]
mod test{
    use super::*;
    
    #[test]
    fn test_log_secret() {
        assert_eq!(format_secret(""), "**");
        assert_eq!(format_secret("1"), "**");
        assert_eq!(format_secret("123"), "1**");
        assert_eq!(format_secret("123456"), "1*****");
        assert_eq!(format_secret("1234567"), "12***67");
        assert_eq!(format_secret("123456789"), "12*****89");
    }
}