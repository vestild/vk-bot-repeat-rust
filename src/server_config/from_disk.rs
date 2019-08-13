use std::fs::{File, OpenOptions};
use std::io::{Result, Seek, SeekFrom};
use log::{info, error};

use crate::config;
use super::{ServerConfig};

struct ConfigFile(Option<File>);

const FILE_SIZE: u64 = 1024;

impl ConfigFile {
    fn open() -> ConfigFile {
        let filename = match config::CONF.server_options_file() {
            Some(name) => {
                info!("use file {}", name);
                name
            },
            None => return ConfigFile(None)
        };
        
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename);
        
        match file {
            Err(e) => {
                error!("can't open file {}", e);
                ConfigFile(None)
            },
            Ok(f) => {
                match set_len(&f) {
                    Err(e) => {
                        error!("can't set file size {}", e);
                        ConfigFile(None)
                    },
                    Ok(_) => ConfigFile(Some(f))
                }
            }
        }
    }
    
    fn get(&mut self) -> Option<ServerConfig> {
        self.0.as_ref().and_then(|f| 
            read_config(f)
            .map_err(|e| error!("can't read from file {}", e))
            .ok())
    }
    
    fn set(&mut self, config: &ServerConfig) {
        self.0.as_ref().and_then(|f| 
            write_config(f, config)
            .map_err(|e| error!("can't write to file {}", e))
            .ok());
    }
}

fn set_len(file: &File) -> Result<()> {
    let len = file.metadata()?.len();
    if len < FILE_SIZE {
        file.set_len(FILE_SIZE)
    } else {
        Ok(())
    }
}

fn read_config(mut file: &File) -> Result<ServerConfig> {
    file.seek(SeekFrom::Start(0))?;
    let config = serde_json::from_reader(file)?;
    Ok(config)
}

fn write_config(mut file: &File, config: &ServerConfig) -> Result<()> {
    file.seek(SeekFrom::Start(0))?;
    serde_json::to_writer(file, config).map_err(|e| e.into())
}