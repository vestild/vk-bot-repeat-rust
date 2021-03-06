use crate::client::{Client, ServerConfig};
use crate::error::*;
use log::{debug, error};
use std::io::SeekFrom;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, AsyncWriteExt, BufReader, Result};

pub struct ConfigProvider(File);

const FILE_SIZE: u64 = 1024;

pub async fn write(provider: &mut Option<ConfigProvider>, config: &ServerConfig) -> () {
    if let Some(p) = provider {
        let r = write_config(&mut p.0, config).await;
        if let Err(e) = r {
            error!("{}", e);
        }
    }
}

pub async fn with_file(
    client: &Client,
    file_name: &str,
) -> SimpleResult<(ConfigProvider, ServerConfig)> {
    let (mut file, config) = open(file_name).await?;
    let config = match config {
        Some(c) => c,
        None => {
            let c = client.long_poll_config().await?;
            write_config(&mut file, &c).await.wrap_err("can't write")?;
            c
        }
    };
    Ok((ConfigProvider(file), config))
}

async fn open(file_name: &str) -> SimpleResult<(File, Option<ServerConfig>)> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_name)
        .await
        .wrap_err("can't open")?;
    let config = read_config(&mut file).await.wrap_err("can't read config")?;
    if config.is_none() {
        set_len(&mut file).await.wrap_err("can't set len")?
    }
    Ok((file, config))
}

async fn read_config(file: &mut File) -> Result<Option<ServerConfig>> {
    file.seek(SeekFrom::Start(0)).await?;
    let mut line = String::new();
    BufReader::new(file).read_line(&mut line).await?;
    match serde_json::from_str(&line) {
        Ok(c) => {
            debug!("read {:?}", line);
            Ok(Some(c))
        }
        _ => Ok(None),
    }
}

async fn set_len(file: &mut File) -> Result<()> {
    let len = file.metadata().await?.len();
    if len < FILE_SIZE {
        file.set_len(FILE_SIZE).await?;
    }
    Ok(())
}

async fn write_config(file: &mut File, config: &ServerConfig) -> Result<()> {
    let l = serde_json::to_string(config).unwrap() + "\n";
    debug!("write {:?}", l);
    file.seek(SeekFrom::Start(0)).await?;
    file.write_all(l.as_bytes()).await?;
    Ok(())
}
