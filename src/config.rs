use std::{collections::HashMap, ffi::OsString, fmt, fs};

use anyhow::{Context, Result};
use serde::Deserialize;

pub enum ConfigPath {
    Default,
    Provided(OsString),
}

impl ConfigPath {
    pub fn read(&self) -> Result<Config> {
        let config = match self {
            ConfigPath::Default => fs::read_to_string("Spa.toml")
                .context("could not find a `Spa.toml` in the current directory")?,
            ConfigPath::Provided(path) => fs::read_to_string(&path).with_context(|| {
                format!("could not read config file at {}", path.to_string_lossy())
            })?,
        };
        toml::from_str::<Config>(&config)
            .with_context(|| format!("`{}` is not a valid config file", self))
    }
}

impl fmt::Display for ConfigPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigPath::Default => write!(f, "Spa.toml"),
            ConfigPath::Provided(path) => write!(f, "{}", path.to_string_lossy()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub proxies: HashMap<String, ProxyTarget>,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub folder: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: String,
}

#[derive(Debug, Deserialize)]
pub struct ProxyTarget {
    pub target: String,
    #[serde(default)]
    pub path_rewrite: Option<(String, String)>,
    #[serde(default)]
    pub secure: bool,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

fn default_port() -> u16 {
    4300
}
fn default_host() -> String {
    "127.0.0.1".to_owned()
}
