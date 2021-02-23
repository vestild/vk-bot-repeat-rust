use crate::client::{Client, ServerConfig};
use crate::config;
use crate::error::*;
use crate::long_poll_client::{get_events, Event, Result};
use crate::server_config::{write, ConfigProvider};
use ctrlc;
use futures::{future::FutureExt, pin_mut, select};
use futures_intrusive::sync::ManualResetEvent;
use log::error;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub struct Worker {
    group_id: u64,
    chat_id: i64,
    client: Client,
    config: ServerConfig,
    config_provider: Option<ConfigProvider>,
    stopper: Stopper,
    last_error: bool,
}

impl Worker {
    pub fn new(client: Client, config: ServerConfig, provider: Option<ConfigProvider>) -> Worker {
        Worker {
            group_id: config::group_id(),
            chat_id: config::chat_peer_id(),
            client,
            config,
            config_provider: provider,
            stopper: Stopper::new(),
            last_error: false,
        }
    }

    pub async fn main_loop(&mut self) {
        let raw_client = self.client.raw_client();
        let stopper = self.stopper.clone();
        while !self.stopper.stopped() {
            let s = stopper.wait().fuse();
            let p = self.process_events(&raw_client).fuse();

            pin_mut!(p, s);

            select! {
                _ = p => (),
                _ = s => return,
            }
        }
    }

    async fn process_events(&mut self, raw_client: &reqwest::Client) {
        let r = get_events(&raw_client, &self.config).await;
        match r {
            Err(e) => {
                self.handle_error(&e).await;
            }
            Ok(result) => {
                self.last_error = false;
                self.handle_events(&result.events).await;
                self.handle_config(result).await;
            }
        }
    }

    async fn handle_result<T>(&mut self, r: &SimpleResult<T>) {
        if let Err(e) = r {
            self.handle_error(e).await
        }
    }

    async fn handle_error(&mut self, e: &Error) {
        error!("Error: {}", e);
        let sleep_seconds = if self.last_error { 15 } else { 5 * 60 };
        self.last_error = true;
        let w = sleep(Duration::new(sleep_seconds, 0)).fuse();
        let s = self.stopper.wait().fuse();

        pin_mut!(w, s);

        select! {
            _ = w => (),
            _ = s => (),
        }
    }

    async fn handle_config(&mut self, result: Result) {
        if !result.refresh_all && !result.refresh_key {
            if let Some(ts) = result.ts {
                if ts != self.config.ts {
                    self.config.ts = ts;
                    self.write_config().await;
                }
            }
            return;
        }

        match self.client.long_poll_config().await {
            Err(e) => self.handle_error(&e).await,
            Ok(new_config) => {
                if result.refresh_all {
                    self.config = new_config
                } else {
                    self.config.key = new_config.key
                }

                if let Some(ts) = result.ts {
                    self.config.ts = ts
                }
                self.write_config().await;
            }
        }
    }

    async fn write_config(&mut self) {
        write(&mut self.config_provider, &self.config).await;
    }

    async fn handle_events(&mut self, events: &[Event]) {
        for event in events {
            match event {
                Event::WallPost { id } => {
                    let attachment = format!("wall-{}_{}", self.group_id, id);
                    let r = self
                        .client
                        .send_message(self.chat_id, None, Some(attachment))
                        .await;
                    self.handle_result(&r).await;
                }
                Event::BoardPost {
                    from_id,
                    text,
                    topic_id,
                    id,
                } => {
                    let mut text = text.to_owned();
                    text.truncate(100);
                    let user = self.client.get_user(*from_id).await;
                    let user_name = match user {
                        Err(e) => {
                            self.handle_error(&e).await;
                            String::new()
                        }
                        Ok(user) => format!(
                            "{} {}",
                            user.first_name.unwrap_or(String::new()),
                            user.last_name.unwrap_or(String::new())
                        ),
                    };
                    let message = format!(
                        "{}: {} \n https://vk.com/topic-{}_{}?post={}",
                        user_name, text, self.group_id, topic_id, id
                    );
                    let r = self
                        .client
                        .send_message(self.chat_id, Some(message), None)
                        .await;
                    self.handle_result(&r).await;
                }
            }
        }
    }
}

#[derive(Clone)]
struct Stopper(Arc<ManualResetEvent>);

impl Stopper {
    fn new() -> Stopper {
        let running = Arc::new(ManualResetEvent::new(false));
        let r = running.clone();
        ctrlc::set_handler(move || r.set()).expect("Error setting Ctrl-C handler");
        Stopper(running)
    }

    fn stopped(&self) -> bool {
        self.0.is_set()
    }

    async fn wait(&self) {
        self.0.wait().await
    }
}
