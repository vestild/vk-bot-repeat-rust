#![recursion_limit="1024"]
#[macro_use]

mod client;
mod long_poll_client;
mod config;
mod server_config;
mod mask_secret;
mod error;
mod worker;

use simplelog::*;
use log::{info};
use server_config::ConfigProvider;
use client::{Client, ServerConfig};
use error::SimpleResult;

#[tokio::main(basic_scheduler)]
async fn main() {
    SimpleLogger::init(LevelFilter::Debug, Config::default()).unwrap();
    let client = new_client();
    let (p, c) = new_provider(&client).await.unwrap();
    worker::Worker::new(client, c, p).main_loop().await;
}

fn new_client() -> Client {
    let token = config::token();
    info!(target: "main", "token {:?}", mask_secret::mask(&token));
    client::Client::new(token, config::group_id())
}

async fn new_provider(client: &Client) -> SimpleResult<(Option<ConfigProvider>, ServerConfig)> {
    match config::server_options_file() {
        Some(file_name) => {
            let (p, c) = server_config::with_file(client, &file_name,).await?;
            Ok((Some(p), c))
        },
        None => {
            let c = client.long_poll_config().await?;
            Ok((None, c))
        }
    }
}