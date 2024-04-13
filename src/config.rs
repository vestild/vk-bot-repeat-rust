use std::env;

fn get_opt(name: &str) -> Option<String> {
    env::var_os(name).and_then(|s| s.to_str().map(|s| s.to_string()))
}

fn get(name: &str) -> String {
    match env::var_os(name) {
        Some(val) => match val.to_str() {
            Some(val) => val.into(),
            None => panic!("Bad value in enviroment variable {} not present", name),
        },
        None => panic!("Enviroment variable {} not present", name),
    }
}

pub fn token() -> String {
    get("VK_BOT_TOKEN")
}

pub fn group_id() -> u64 {
    get("VK_BOT_GROUP").parse().expect("not int GROUP")
}

pub fn chat_peer_id() -> i64 {
    get("VK_BOT_CHAT").parse().expect("not int CHAT")
}

pub fn server_options_file() -> Option<String> {
    get_opt("VK_BOT_FILE")
}
