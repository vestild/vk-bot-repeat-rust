#![recursion_limit = "1024"]
#[macro_use]

mod client;
mod config;
mod error;
mod long_poll_client;
mod mask_secret;
mod server_config;
mod worker;

use client::{Client, ServerConfig};
use error::SimpleResult;
use log::info;
use server_config::ConfigProvider;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();
    info!("start bot");
    let client = new_client();
    let (p, c) = new_provider(&client).await.unwrap();
    let ct = tokio_util::sync::CancellationToken::new();
    let w = tokio::spawn(worker::run(client, c, p, ct.clone()));
    match tokio::signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
            // we also shut down in case of error
        }
    }
    ct.cancel();
    w.await.unwrap()
}

fn new_client() -> Client {
    let token = config::token();
    info!(target: "main", "token {:?}", mask_secret::mask(&token));
    client::Client::new(token, config::group_id())
}

async fn new_provider(client: &Client) -> SimpleResult<(Option<ConfigProvider>, ServerConfig)> {
    match config::server_options_file() {
        Some(file_name) => {
            let (p, c) = server_config::with_file(client, &file_name).await?;
            Ok((Some(p), c))
        }
        None => {
            let c = client.long_poll_config().await?;
            Ok((None, c))
        }
    }
}
