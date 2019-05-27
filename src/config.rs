use std::env;

pub struct Config {
    token: String,
}

impl Config {
    pub fn token(&self) -> &str {
        &self.token[..]
    }
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
        token: get ("VK_BOT_TOKEN")
    }
}

lazy_static! {
    pub static ref CONF: Config = load();
}

