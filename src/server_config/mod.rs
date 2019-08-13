use serde::{Deserialize, Serialize};

mod from_vk;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    key: String,
    server: String,
    ts: String,
}