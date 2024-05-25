use std::{env, io::ErrorKind};

use serde::{Deserialize, Serialize};
use tokio::fs;

/// The configuration of the application.
#[derive(Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub public_config: PublicConfig,
}

/// The publicly visible part of the config. Used for settings that also affect the frontend.
#[derive(Deserialize, Serialize, Clone)]
pub struct PublicConfig {
    /// Allows new accounts to be registered on the site.
    #[serde(default)]
    pub allow_registering: bool,
}

impl Config {
    /// Read the configuration file.
    pub async fn read() -> anyhow::Result<Self> {
        let config_path =
            env::var("BEACON_SERVER_CONFIG").unwrap_or_else(|_| "beacon.toml".to_string());

        let config_str = match fs::read_to_string(config_path).await {
            Ok(v) => v,
            Err(err) if err.kind() == ErrorKind::NotFound => String::new(),
            Err(err) => return Err(err.into()),
        };

        Ok(toml::from_str(&config_str)?)
    }
}
