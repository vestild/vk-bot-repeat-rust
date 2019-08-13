use log::{info, error};
use reqwest::r#async::{Client};
use futures::{Future};
use serde::{Deserialize, Serialize};

use crate::config;
use super::{ServerConfig};

#[derive(Debug, Serialize, Deserialize)]
struct ResponseWrapper<T> {
    response: Option<T>,
    error: Option<Error>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Error {
    error_code: u64,
    error_msg: String,
}

fn unwrap_response(resp: ResponseWrapper<ServerConfig>) -> Result<ServerConfig, String> {
    match resp.response {
        Some(r) => Ok(r),
        None => match resp.error {
            None => Err("bad response".to_string()),
            Some(e) => {
                error!("Error response: {:?}", e);
                Err(e.error_msg)
            }
        }
    }
}

pub fn get_new() -> impl Future<Item=ServerConfig, Error=String> {
    let uri = format!("https://api.vk.com/method/groups.getLongPollServer?group_id={}&access_token={}&v=5.100",
                      config::CONF.group_id(), config::CONF.token());

    Client::new()
        .get(&uri)
        .send()
        .and_then(|mut res| {
            info!("status {}", res.status());

            res.json::<ResponseWrapper<ServerConfig>>()
        })
        .then(|res| {
            match res {
                Ok(resp) => unwrap_response(resp),
                Err(err) => {
                    error!("Error: {}", err);
                    Err(format!("{}", err))
                }
            }
        })
}