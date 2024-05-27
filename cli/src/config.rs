use std::{env, path::Path};

use anyhow::Context;
use serde::Deserialize;
use tokio::fs;

#[derive(Deserialize)]
pub struct Config {
    /// The URL of the host where files should be pushed through. Should include `https://`.
    pub host: String,
    /// The username to use to authenticate with.
    pub username: String,
    /// The location to the user's private SSH key.
    ///
    /// If unset, this will loop over the `~/.ssh` directory and use the first SSH key it finds.
    pub ssh_key: Option<String>,
}

impl Config {
    /// Read and parse the configuration file.
    pub async fn read() -> anyhow::Result<Self> {
        let config_path =
            env::var("BEACON_CLI_CONFIG").unwrap_or_else(|_| "~/beacon.toml".to_string());
        // Expand the path (such as replacing ~ with the value of $HOME).
        let config_path = shellexpand::full(&config_path).context(
            "could not expand config path, `BEACON_CLI_CONFIG` may have been set incorrectly",
        )?;

        let config_str = fs::read_to_string(Path::new(config_path.as_ref()))
            .await
            .context(format!("could not read config file at `{config_path}`"))?;
        toml::from_str(&config_str).context(format!("error parsing config file at `{config_path}`"))
    }
}
