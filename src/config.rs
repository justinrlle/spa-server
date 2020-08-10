use std::{collections::HashMap, ffi::OsString, fmt, fs};

use anyhow::{Context, Result};
use serde::Deserialize;

pub enum ConfigPath {
    Default,
    Provided(OsString),
}

impl ConfigPath {
    const DEFAULT_PATH: &'static str = "Spa.toml";
    pub fn read(&self) -> Result<Config> {
        if let ConfigPath::Provided(path) = self {
            debug!("loading config from `{}`", path.to_string_lossy());
        } else {
            debug!(
                "loading config from default path `{}`",
                ConfigPath::DEFAULT_PATH
            )
        }
        let config = match self {
            ConfigPath::Default => {
                fs::read_to_string(ConfigPath::DEFAULT_PATH).with_context(|| {
                    format!(
                        "could not find a `{}` in the current directory",
                        ConfigPath::DEFAULT_PATH
                    )
                })?
            }
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
    #[serde(default)]
    pub base_path: Option<String>,
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
    4242
}
fn default_host() -> String {
    "127.0.0.1".to_owned()
}
