#[macro_use]
extern crate log;
use std::path::PathBuf;
use std::{borrow::Cow, env, fs};

use anyhow::{Context, Result};
use argh::FromArgs;
use log::LevelFilter;

use config::ConfigPath;
use server::Server;

mod config;
mod server;
mod source;

#[derive(Debug, FromArgs)]
/// spa-server, a local server for already built SPAs (Single Page Applications).
struct Options {
    /// log level of the application, defaults to `WARN`, can be one of `OFF`, `ERROR`, `WARN`, `INFO`, `DEBUG`, `TRACE`
    #[argh(option, short = 'l', default = "LevelFilter::Warn")]
    log: LevelFilter,
    /// serve from a folder instead of reading from config
    #[argh(option, short = 's')]
    serve: Option<String>,
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
            let date_time = chrono::Utc::now().with_timezone(&offset);
            out.finish(format_args!(
                "{level:<5} [{time}] {target}: {message}",
                level = colors.color(record.level()),
                time = date_time.format("%Y-%m-%d %H:%M:%S%.3f"),
                target = record.target(),
                message = message
            ))
        })
        .level(LevelFilter::Error)
        .level_for("spa_server", level)
        .level_for(
            "spa_server::server::proxy",
            std::cmp::max(LevelFilter::Warn, level),
        )
        .chain(std::io::stderr())
        .apply()?;
    Ok(())
}

fn main() -> Result<()> {
    let opts: Options = argh::from_env();
    setup_logger(opts.log).context("failed to init logger, this is surely a bug")?;
    trace!("options: {:#?}", opts);
    let config = if let Some(folder) = &opts.serve {
        trace!("using serve option instead of config file");
        config::from_folder(folder.to_owned())
    } else {
        let config_location = opts
            .config
            .map(ConfigPath::Provided)
            .unwrap_or(ConfigPath::Default);
        config_location.read()?
    };

    load_env_file(opts.env_file.as_deref())?;

    let app_path = expand_path(&config.server.serve)?;
    let source = source::detect(&app_path);
    let cache_folder = &cache_dir()?;
    let folder = source.setup(&app_path, cache_folder, config.server.base_path.as_deref())?;

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

fn load_env_file(opt_env_file: Option<&str>) -> Result<()> {
    if let Some(env_file) = opt_env_file {
        trace!("loading env file from {}", env_file);
        dotenv::from_path(env_file)
            .with_context(|| format!("failed to load env file at `{}`", env_file))?;
    } else {
        trace!("loading .env");
        dotenv::dotenv().ok();
    };
    Ok(())
}
