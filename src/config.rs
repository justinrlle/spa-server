use std::{collections::HashMap, fmt, fs};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    /// Configure the server
    pub server: ServerConfig,
    /// Configure the proxies. The keys represent the part that will be matched to test if a call
    /// must be proxied, and they will always be matched at the beginning of the request's url.
    pub proxies: HashMap<String, ProxyTarget>,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    /// The only necessary property, determining what will be served. It can be a path to a folder,
    /// a path to an archive, or an http url pointing to an archive. It can contain the `~` and
    /// environment variables.
    pub serve: String,
    /// The base path to use inside the application, only used (and useful) with archive or http
    /// resources.
    /// # Example
    /// ```toml
    /// [server]
    /// serve = "~/app.tar.gz"
    /// base_path = "app"
    /// ```
    #[serde(default)]
    pub base_path: Option<String>,
    /// The port the application should listen on, defaults to [default_port](ServerConfig::default_port)
    #[serde(default = "ServerConfig::default_port")]
    pub port: u16,
    /// The host the application should listen on, defaults to [default_host](ServerConfig::default_host)
    #[serde(default = "ServerConfig::default_host")]
    pub host: String,
}

impl ServerConfig {
    fn default_port() -> u16 {
        4242
    }
    fn default_host() -> String {
        "127.0.0.1".to_owned()
    }
}

/// Currently, a proxy target can only be defined as a path to be matched, and an url to send the
/// same request to. No path rewrite is supported at all.
#[derive(Debug, Deserialize)]
pub struct ProxyTarget {
    /// The target url (protocol, host, port, paths...).
    pub target: String,
    /// Path rewriting, not used for now.
    #[serde(default)]
    pub path_rewrite: Option<(String, String)>,
    /// Headers to add to the proxied request.
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

pub fn from_folder(folder: String) -> Config {
    Config {
        server: ServerConfig {
            serve: folder,
            base_path: None,
            host: ServerConfig::default_host(),
            port: ServerConfig::default_port(),
        },
        proxies: HashMap::new(),
    }
}

pub enum ConfigPath {
    Default,
    Provided(String),
}

impl ConfigPath {
    const DEFAULT_PATH: &'static str = "Spa.toml";
    pub fn read(&self) -> Result<Config> {
        if let ConfigPath::Provided(path) = self {
            debug!("loading config from `{}`", path);
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
            ConfigPath::Provided(path) => fs::read_to_string(&path)
                .with_context(|| format!("could not read config file at {}", path))?,
        };
        toml::from_str::<Config>(&config)
            .with_context(|| format!("`{}` is not a valid config file", self))
    }
}

impl fmt::Display for ConfigPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigPath::Default => write!(f, "Spa.toml"),
            ConfigPath::Provided(path) => write!(f, "{}", path),
        }
    }
}
