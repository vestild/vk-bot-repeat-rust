use serde::{Deserialize, Serialize};
use futures::{Future};
use futures::future::Either;

mod from_vk;
mod from_disk;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    key: String,
    server: String,
    ts: String,
}

pub struct ConfigProvider {
    disk_writer: from_disk::ConfigFile,
}

impl ConfigProvider {
    pub fn new() -> (ConfigProvider, Option<ServerConfig>) {
        let (disk_writer, config) = from_disk::ConfigFile::open();
        (ConfigProvider{disk_writer}, config)
    }

    pub fn reset(self) -> impl Future<Item=(ConfigProvider, ServerConfig), Error=String> {
        let disk = self.disk_writer.clone();
        from_vk::get_new()
            .map(move |c| {
                disk.set(&c);
                (self, c)
            })
    }
}