use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use reqwest::{Response};
use crate::error::{*};
use simple_error::SimpleError;

pub struct Client {
    client: reqwest::Client,
    url: String,
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseWrapper<T> {
    response: Option<T>,
    error: Option<ErrorDescription>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorDescription {
    error_code: u64,
    error_msg: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub key: String,
    pub server: String,
    pub ts: String,
}

impl Client {
    pub fn new(token: String) -> Client {
        Client {
            client: reqwest::Client::new(),
            url: "https://api.vk.com/method/".to_string(),
            token,
        }
    }

    pub fn internal_client(&self) -> reqwest::Client {
        self.client.clone()
    }

    pub async fn long_poll_config(&self, group_id: u64) -> SimpleResult<ServerConfig> {
        let query = [
            ("v", "5.100"),
            ("group_id", &group_id.to_string()),
            ("access_token", &self.token)
        ];
        send(self, "groups.getLongPollServer", &query).await
        // groups.getLongPollServer?group_id={}&access_token={}&v=5.100
    }
}

async fn send<T: DeserializeOwned, TQuery: Serialize + ?Sized>(client: &Client, method: &str, query: &TQuery) -> SimpleResult<T> {
    let url: String = client.url.clone() + method;
    let r: Response = client.client.get(&url)
        .query(query)
        .send()
        .await
        .map_err(|e| wrap(&e, method))?;
    let status = r.status();
    let text = r.text().await.map_err(|e| wrap(&e, method))?;
    if !status.is_success() {
        return Err(Error::new(format!("got status {} on send {} with body {}", status, method, &text)))
    }
    let r: ResponseWrapper<T> = serde_json::from_str(&text)
        .map_err(|e| Error::new(format!("{:?} on deserialize <{}> from {}", e, &text, method)))?;
    if r.error.is_some() || !r.response.is_some() {
        return Err(Error::new(format!("got <{}> from {}", &text, method)))
    }
    Ok(r.response.unwrap())
}

fn wrap(e: &reqwest::Error, method: &str) -> SimpleError {
    Error::new(format!("got {:?} from {}", e, method))
}

