use std::{ffi::OsString, fmt, fs, collections::HashMap};

use anyhow::{Context, Result};
use rouille::url;
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

#[derive(Debug)]
pub struct ProxyConfig {
    pub path: String,
    pub target: String,
    pub path_rewrite: Option<(String, String)>,
    pub secure: bool,
    pub headers: HashMap<String, String>,
}



impl ProxyConfig {
    pub fn new(path: &str, proxy: &ProxyTarget) -> Result<Self> {
        anyhow::ensure!(
            path.starts_with('/'),
            "path `{}` is not a valid path",
            path
        );
        let mut path = path.to_owned();
        if !path.ends_with('/') {
            path += "/";
        }
        url::Url::parse(&proxy.target).context("invalid target")?;
        let target = proxy.target.clone();
        let path_rewrite = proxy.path_rewrite.clone();
        let secure = proxy.secure;
        let headers = proxy.headers.clone();
        Ok(Self {
            path, target, path_rewrite, secure, headers
        })
    }
}

fn default_port() -> u16 {
    4300
}
fn default_host() -> String {
    "127.0.0.1".to_owned()
}
