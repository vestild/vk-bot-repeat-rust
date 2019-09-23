use log::{info, error};
use reqwest::r#async::{Client};
use futures::{Future};
use serde::{Deserialize, Serialize};

use crate::config;
use crate::server_config::{ConfigProvider, ServerConfig};
use futures::future::{Either, IntoFuture};

#[derive(Debug, Serialize, Deserialize)]
struct ResponseWrapper<T> {
    response: Option<T>,
    error: Option<Error>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LongPollResponse {
    key: String,
    server: String,
    ts: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Error {
    error_code: u64,
    error_msg: String,
}

pub fn run(config: (ConfigProvider, Option<ServerConfig>)) -> impl Future<Item=(), Error=()> {
    init_config(config)
        .and_then(request)
        .map_err(|e| error!("error {}", e))
}

fn init_config((provider, config): (ConfigProvider, Option<ServerConfig>))
    -> impl Future<Item=(ConfigProvider, ServerConfig), Error=String> {
    match config {
        Some(c) => Either::A(Ok((provider, c)).into_future()),
        None => Either::B(provider.reset())
    }
}

fn request((provider, config): (ConfigProvider, ServerConfig)) -> impl Future<Item=(), Error=String> {
    Ok(()).into_future()
}