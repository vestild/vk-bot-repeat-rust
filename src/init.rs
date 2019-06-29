use std::mem;
use log::{info, error};
use reqwest::r#async::{Client, Decoder};
use futures::{Future, Stream};
use serde::{Deserialize, Serialize};

use crate::config;

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

pub fn run() -> impl Future<Item=(), Error=()> {
    let uri = format!("https://api.vk.com/method/groups.getLongPollServer?group_id={}&access_token={}&v=5.100",
                      config::CONF.group_id(), config::CONF.token());

    Client::new()
        .get(&uri)
        .send()
        .and_then(|mut res| {
            info!("status {}", res.status());

            res.json::<ResponseWrapper<LongPollResponse>>()
        })
        .map(|resp| {
            info!("resp {:#?}", resp);
        })
        .map_err(|err| {
            error!("Error: {}", err);
        })
}