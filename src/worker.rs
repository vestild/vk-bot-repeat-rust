use crate::client::{Client, ServerConfig};
use crate::config;
use crate::error::*;
use crate::long_poll_client::{get_events, Event, Result};
use crate::server_config::{write, ConfigProvider};
use log::error;
use std::time::Duration;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

struct Worker {
    group_id: u64,
    chat_id: i64,
    client: Client,
    config: ServerConfig,
    config_provider: Option<ConfigProvider>,
    last_error: bool,
    cancelation: CancellationToken,
}

pub async fn run(
    client: Client,
    config: ServerConfig,
    provider: Option<ConfigProvider>,
    ct: CancellationToken,
) {
    let mut w = Worker {
        group_id: config::group_id(),
        chat_id: config::chat_peer_id(),
        client,
        config,
        config_provider: provider,
        last_error: false,
        cancelation: ct,
    };

    w.main_loop().await
}

impl Worker {
    pub async fn main_loop(&mut self) {
        let raw_client = self.client.raw_client();
        let ct = self.cancelation.clone();
        loop {
            tokio::select! {
                _ = ct.cancelled() => { return },
                _ = self.process_events(&raw_client) => ()
            }
        }
    }

    async fn process_events(&mut self, raw_client: &reqwest::Client) {
        let r = get_events(raw_client, &self.config).await;
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
        let ct = self.cancelation.clone();
        tokio::select! {
          _ = sleep(Duration::new(sleep_seconds, 0)) => (),
          _ = ct.cancelled() => () ,
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
                    let text = text.chars().take(100).collect::<String>();
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
