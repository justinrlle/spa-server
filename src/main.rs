#[macro_use]
extern crate log;
use std::path::PathBuf;
use std::{borrow::Cow, env, fs};

use anyhow::{Context, Result};
use argh::FromArgs;
use log::LevelFilter;

use config::ConfigPath;
use server::Server;

mod archive;
mod config;
mod server;

#[derive(Debug, FromArgs)]
/// spa-server, a local server for already built SPAs (Single Page Applications).
struct Options {
    /// log level of the application, defaults to `WARN`, can be one of `OFF`, `ERROR`, `WARN`, `INFO`, `DEBUG`, `TRACE`
    #[argh(option, short = 'l', default = "LevelFilter::Warn")]
    log: LevelFilter,
    /// optional `dotenv` file with variables needed for path url
    #[argh(option, short = 'e')]
    env_file: Option<String>,
    /// path to config file, defaults to `Spa.toml`
    #[argh(positional)]
    config: Option<String>,
}

fn setup_logger(level: LevelFilter) -> Result<()> {
    let offset = chrono::Local::now().offset().to_owned();
    use fern::colors::{Color, ColoredLevelConfig};
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::Blue)
        .trace(Color::Magenta);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{level:<5} [{time}] {target}: {message}",
                level = colors.color(record.level()),
                time = chrono::Utc::now().with_timezone(&offset).format("%Y-%m-%d %H:%M:%S%.3f"),
                target = record.target(),
                message = message
            ))
        })
        .level(LevelFilter::Error)
        .level_for("spa_server::server::proxy", LevelFilter::Warn)
        .level_for("spa_server", level)
        .chain(std::io::stderr())
        .apply()?;
    Ok(())
}

fn main() -> Result<()> {
    let opts: Options = argh::from_env();
    setup_logger(opts.log).context("failed to init logger, this is surely a bug")?;
    trace!("options: {:#?}", opts);
    let config_location = opts
        .config
        .map(ConfigPath::Provided)
        .unwrap_or(ConfigPath::Default);
    let config = config_location.read()?;

    let _env_file = opts.env_file;

    let folder = expand_path(&config.server.folder)?;
    let archive = archive::detect(&folder);

    let folder = if let Some(archive) = archive {
        anyhow::ensure!(
            archive.is_tar(),
            "got {:?} archive, only tar archives are supported",
            archive.kind()
        );
        let cache_folder = cache_dir()?;
        let extracted_path = archive::extract(&folder, archive, &cache_folder)?;
        info!("serving from archive at {}", &config.server.folder);
        if let Some(base_folder) = &config.server.base_path {
            let mut extracted_path = extracted_path;
            extracted_path.push(base_folder);
            extracted_path
        } else {
            extracted_path
        }
    } else {
        info!("serving from folder at {}", &config.server.folder);
        PathBuf::from(folder.into_owned())
    };

    debug!("serving from: {}", folder.display());

    let server = Server::new(folder, &config.proxies)?;
    debug!("proxies: {:?}", server.proxies);

    let addr = (config.server.host.as_ref(), config.server.port);

    let server = rouille::Server::new(addr, move |request| {
        rouille::log_custom(request, server::log_success, server::log_error, || {
            server.serve_request(request)
        })
    })
    .map_err(|e| anyhow::anyhow!(e))
    .with_context(|| format!("Failed to listen on port {}", addr.1))?
    .pool_size(8 * num_cpus::get());

    println!("Listening on http://{}", server.server_addr());
    server.run();
    Ok(())
}

fn expand_path(path: &str) -> Result<Cow<str>> {
    let expanded = shellexpand::full_with_context(path, dirs::home_dir, |s| {
        if let Some(pos) = s.find("::") {
            let namespace = &s[..pos];
            let key = &s[pos + 2..];
            // TODO: decide how to handle secrets and env
            Ok(Some(format!("`namespace={},key={}`", namespace, key)))
        } else {
            std::env::var(s).map(Some)
        }
    })
    .with_context(|| format!("failed to expand path: {}", path))?;
    Ok(expanded)
}

fn cache_dir() -> Result<PathBuf> {
    let temp_dir = env::temp_dir();
    let project_dirs = directories::ProjectDirs::from("beer", "justinrlle", "Spa-Server");
    let cache_folder = project_dirs
        .map(|p| p.cache_dir().to_owned())
        .unwrap_or_else(|| temp_dir.clone());
    debug!("cache folder: {}", cache_folder.display());
    fs::create_dir_all(&cache_folder)
        .with_context(|| format!("failed to create cache path: {}", cache_folder.display()))?;
    Ok(cache_folder)
}
