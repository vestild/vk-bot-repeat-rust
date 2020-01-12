use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use ctrlc;
use log::{error};
use crate::server_config::ConfigProvider;
use crate::error::{*};
use crate::client::{ServerConfig, Client};

pub async fn main_loop(client: Client, mut config: ServerConfig, mut provider: Option<ConfigProvider>) {
    let stopper = handle_stop();

    while stopper.load(Ordering::SeqCst) {

    }
}

fn handle_stop() -> Arc<AtomicBool> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    running
}