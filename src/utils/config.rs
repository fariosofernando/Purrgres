use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use super::path::get_bkp_path;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub remote: Option<RemoteConfig>,
    pub server: Option<ServerConfig>,
    pub retention: Option<RetentionConfig>,
}

#[derive(Debug, Deserialize)]
pub struct RemoteConfig {
    #[serde(default)]
    pub enabled: bool,
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub api_key: String,
    #[serde(default = "default_true")]
    pub compress: bool,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_storage_path")]
    pub storage_path: PathBuf,
    pub api_key: String,
    #[serde(default = "default_max_upload_mb")]
    pub max_upload_size_mb: u64,
    pub max_remote_backups: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct RetentionConfig {
    #[serde(default = "default_max_local")]
    pub max_local_backups: u32,
}

fn default_max_local() -> u32 {
    7
}

fn default_port() -> u16 {
    8443
}

fn default_true() -> bool {
    true
}

fn default_storage_path() -> PathBuf {
    get_bkp_path().join("remote_backups")
}

fn default_max_upload_mb() -> u64 {
    500
}

impl Config {
    /// Loads config from a given path, or falls back to ~/.purrgres/purrgres.toml
    pub fn load(path: Option<&Path>) -> Result<Self, String> {
        let config_path = match path {
            Some(p) => p.to_path_buf(),
            None => get_bkp_path().join("purrgres.toml"),
        };

        if !config_path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(&config_path).map_err(|e| {
            format!(
                "Failed to read config file '{}': {}",
                config_path.display(),
                e
            )
        })?;

        let config: Config = toml::from_str(&content).map_err(|e| {
            format!(
                "Failed to parse config file '{}': {}",
                config_path.display(),
                e
            )
        })?;

        Ok(config)
    }
}

impl ServerConfig {
    pub fn max_upload_bytes(&self) -> u64 {
        self.max_upload_size_mb * 1024 * 1024
    }
}
