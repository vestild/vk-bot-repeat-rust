use std::fs::{File, OpenOptions};
use std::io::{Result, Seek, SeekFrom, Write, BufReader, BufRead};
use log::{debug, info, error};
use std::sync::mpsc::{Sender, channel};
use std::thread;
use crate::config;
use super::{ServerConfig};

#[derive(Clone)]
pub struct ConfigFile(Option<Sender<ServerConfig>>);

const FILE_SIZE: u64 = 1024;

impl ConfigFile {
    pub fn open() -> (ConfigFile, Option<ServerConfig>) {
        let filename = match config::CONF.server_options_file() {
            Some(name) => {
                info!("use file {}", name);
                name
            },
            None => return (ConfigFile(None), None)
        };
        
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename).map_err(|e| {error!("can't open file {}", e); e})
            .and_then(set_len).map_err(|e| error!("can't set file size {}", e));

        match file {
            Err(_) => (ConfigFile(None), None),
            Ok(file) => {
                let config = read_config(&file)
                    .map_err(|e| error!("can't read from file {}", e))
                    .ok();
                let sender = start_writer(file);
                (ConfigFile(Some(sender)), config)
            }
        }
    }

    pub fn set(&self, config: &ServerConfig) {
        if let Some(s) = &self.0 {
            let _ = s.send(config.clone());
        }
    }
}

fn set_len(file: File) -> Result<File> {
    let len = file.metadata()?.len();
    if len < FILE_SIZE {
        file.set_len(FILE_SIZE)?;
    }
    Ok(file)
}

fn read_config(mut file: &File) -> Result<ServerConfig> {
    file.seek(SeekFrom::Start(0))?;
    let mut buf = BufReader::new(file);
    let mut line= String::new();
    buf.read_line(&mut line)?;
    let config = serde_json::from_str(&line)?;
    debug!("read {:?}", config);
    Ok(config)
}

fn start_writer(file: File) -> Sender<ServerConfig> {
    let (send, recv) = channel();
    thread::spawn(move || {
        debug!("start writing thread");
        for config in recv.iter() {
            let _ = write_config(&file, &config)
                .map(|_| debug!("wrote {:?}", config))
                .map_err(|e| error!("can't write to file {}", e));
        }
        debug!("end writing thread");
    });
    send
}

fn write_config(mut file: &File, config: &ServerConfig) -> Result<()> {
    file.seek(SeekFrom::Start(0))?;
    serde_json::to_writer(file, config)
        .map_err(|e| e.into())
        .and_then(|_| write!(file, "\n"))
}