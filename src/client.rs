use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use reqwest::{Response};
use rand::random;
use crate::error::{*};

pub struct Client {
    client: reqwest::Client,
    url: String,
    token: String,
    group_id: u64,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

impl Client {
    pub fn new(token: String, group_id: u64) -> Client {
        Client {
            client: reqwest::Client::new(),
            url: "https://api.vk.com/method/".to_string(),
            token,
            group_id,
        }
    }

    pub fn raw_client(&self) -> reqwest::Client {
        self.client.clone()
    }

    pub async fn long_poll_config(&self) -> SimpleResult<ServerConfig> {
        let query = [
            ("v", "5.100"),
            ("group_id", &self.group_id.to_string()),
            ("access_token", &self.token)
        ];
        send(self, "groups.getLongPollServer", &query).await
    }

    pub async fn send_message(&self, peer_id: i64, text: Option<String>, attachment: Option<String>) -> SimpleResult<()> {
        let peer_id = peer_id.to_string();
        let rand = random::<i64>().abs().to_string();
        let mut query: Vec<(&str, &str)> = vec!(
            ("v", "5.100"),
            ("peer_id", &peer_id),
            ("random_id", &rand),
            ("access_token", &self.token),
        );

        if let Some(text) = &text {
            query.push(("message", text));
        }

        if let Some(attachment) = &attachment {
            query.push(("attachment", attachment));
        }

        let _: i64 = send(self, "messages.send", &query).await?;
        Ok(())
    }

    pub async fn get_user(&self, user_id: i64) -> SimpleResult<User> {
        let user_id = user_id.to_string();
        let query = [
            ("v", "5.100"),
            ("user_ids", &user_id),
            ("access_token", &self.token)
        ];

        let r: SimpleResult<Vec<User>> = send(self, "users.get", &query).await;
        match r {
            Err(e) => Err(e),
            Ok(mut list) => {
                match list.len() {
                    0 => Err(Error::new(format!("user {} not found", user_id))),
                    1 => Ok(list.pop().unwrap()),
                    _ => Err(Error::new(format!("many users: {} found", list.len())))
                }
            }
        }
    }
}

async fn send<T: DeserializeOwned, TQuery: Serialize + ?Sized>(client: &Client, method: &str, query: &TQuery) -> SimpleResult<T> {
    let url: String = client.url.clone() + method;
    let r: Response = client.client.post(&url)
        .form(query)
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

fn wrap(e: &reqwest::Error, method: &str) -> Error {
    Error::new(format!("got {:?} from {}", e, method))
}

