use std::env;

pub struct Config {
    token: String,
    group_id: u64,
    chat_peer_id: u64,
}

impl Config {
    pub fn token(&self) -> &str {
        &self.token[..]
    }
    pub fn group_id(&self) -> u64 { self.group_id }
    pub fn chat_peer_id(&self) -> u64 { self.chat_peer_id }
}

fn get(name: &str) -> String {
    match env::var_os(name) {
        Some(val) => match val.to_str() {
            Some(val) => val.into(),
            None => panic!("Bad value in enviroment variable {} not present", name)
        },
        None => panic!("Enviroment variable {} not present", name)
    }
}

fn load() -> Config {
    Config {
        token: get ("VK_BOT_TOKEN"),
        group_id: get ("VK_BOT_GROUP").parse().expect("not int GROUP"),
        chat_peer_id: get ("VK_BOT_CHAT").parse().expect("not int CHAT"),
    }
}

lazy_static! {
    pub static ref CONF: Config = load();
}

