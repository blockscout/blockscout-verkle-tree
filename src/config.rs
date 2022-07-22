use config::{Config as LibConfig, File};
use serde::{Deserialize};
use std::{net::SocketAddr, str::FromStr, path::PathBuf};


#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct Config {
    pub server: ServerConfiguration,
}

#[derive(Deserialize, Clone)]
#[serde(default)]
pub struct ServerConfiguration {
    pub addr: SocketAddr,
}

impl Default for ServerConfiguration {
    fn default() -> Self {
        Self {
            addr: SocketAddr::from_str("0.0.0.0:8043").expect("should be valid url"),
        }
    }
}

impl Config {
    pub fn from_file(file: PathBuf) -> Result<Self, config::ConfigError> {
        let mut builder = LibConfig::builder();

        if file.exists() {
            builder = builder.add_source(File::from(file));
        }

        builder
            .build()
            .expect("Failed to build config")
            .try_deserialize()
    }
}