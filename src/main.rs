#![recursion_limit="1024"]
#[macro_use]

extern crate lazy_static;

use simplelog::*;
use log::{info};
use futures::future::lazy;

mod config;
mod init;
mod server_config;
mod mask_secret;

#[tokio::main(basic_scheduler)]
async fn main() {
    SimpleLogger::init(LevelFilter::Debug, Config::default()).unwrap();

    info!(target: "main", "token {:?}", mask_secret::mask(config::CONF.token()));
    let sc = server_config::ConfigProvider::new();

    init::run(sc)
}
