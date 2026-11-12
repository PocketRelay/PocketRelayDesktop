use std::{
    net::SocketAddrV4,
    path::{Path, PathBuf},
};

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// Current path to the game executable
    pub game_executable: Option<PathBuf>,
}

/// The config is expected to change in ways that may not make it possible
/// to automatically derive [Default] thus we ignore this warning
#[allow(clippy::derivable_impls)]
impl Default for Config {
    fn default() -> Self {
        Self {
            game_executable: None,
        }
    }
}

pub async fn load_config_from_path(config_path: &Path) -> anyhow::Result<Config> {
    if !config_path.exists() {
        return Ok(Config::default());
    }

    let bytes = tokio::fs::read(config_path)
        .await
        .context("failed to read config file")?;

    let parsed = serde_json::from_slice(&bytes).context("failed to parse config file")?;
    Ok(parsed)
}

pub async fn save_config_to_path(config: &Config, path: &Path) -> anyhow::Result<()> {
    let parent = path
        .parent()
        .context("parent path to config file could not be determined")?;

    if !parent.exists() {
        tokio::fs::create_dir_all(parent)
            .await
            .context("failed to create config parent path")?;
    }

    let serialized = serde_json::to_vec(&config).context("failed to serialize config")?;
    tokio::fs::write(path, serialized)
        .await
        .context("failed to write config file")?;

    Ok(())
}

pub async fn save_rpc_state(game_executable: &Path, rpc_addr: SocketAddrV4) -> anyhow::Result<()> {
    let parent = game_executable
        .parent()
        .context("game executable missing parent path")?;
    let path = parent.join("pocket-relay-rpc");
    tokio::fs::write(path, rpc_addr.to_string()).await?;
    Ok(())
}
