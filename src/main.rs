#[macro_use]
extern crate log;
use std::path::PathBuf;
use std::{
    borrow::Cow,
    env, fs,
    io::{self},
};

use anyhow::{Context, Result};

use config::ConfigPath;
use server::Server;

use crate::archive::ArchiveFormat;
use log::LevelFilter;

mod archive;
mod config;
mod server;

fn setup_logger(level: LevelFilter) -> Result<()> {
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
                "{level:<5} [{target}] {message}",
                level = colors.color(record.level()),
                target = record.target(),
                message = message
            ))
        })
        .level(level)
        .level_for("spa_server::server::proxy", LevelFilter::Warn)
        .chain(std::io::stderr())
        .apply()?;
    Ok(())
}

fn main() -> Result<()> {
    setup_logger(LevelFilter::Debug).context("failed to init logger, this is surely a bug")?;
    let config_location = env::args_os()
        .nth(1)
        .map(ConfigPath::Provided)
        .unwrap_or(ConfigPath::Default);
    let config = config_location.read()?;

    let folder = expand_path(&config.server.folder)?;
    let archive = archive::detect(&folder);

    let folder = if let Some(archive) = archive {
        anyhow::ensure!(
            archive.is_tar(),
            "got {:?} archive, only tar archives are supported",
            archive.kind
        );
        let cache_folder = cache_dir()?;
        let extracted_path = archive::extract_path(&folder, archive, &cache_folder)
            .context("failed to deduce extracted path for archive")?;
        debug!("{}", extracted_path.display());
        extract(&folder, archive, &extracted_path)?;
        if let Some(base_folder) = &config.server.base_path {
            let mut extracted_path = extracted_path;
            extracted_path.push(base_folder);
            extracted_path
        } else {
            extracted_path
        }
    } else {
        PathBuf::from(folder.into_owned())
    };

    debug!("{}", folder.display());

    let server = Server::new(folder, &config.proxies)?;
    debug!("{:?}", server.proxies);

    println!(
        "listening on http://{}:{}",
        config.server.host, config.server.port
    );

    let addr = (config.server.host.as_ref(), config.server.port);

    rouille::start_server_with_pool(addr, None, move |request| {
        rouille::log(request, io::stdout(), || server.serve_request(request))
    });
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
    debug!("{}", cache_folder.display());
    fs::create_dir_all(&cache_folder)
        .with_context(|| format!("failed to create cache path: {}", cache_folder.display()))?;
    Ok(cache_folder)
}

fn extract(folder: &str, archive: ArchiveFormat, extracted_path: &PathBuf) -> Result<()> {
    match fs::metadata(extracted_path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                fs::remove_dir_all(&extracted_path).with_context(|| {
                    format!(
                        "failed to remove old directory: {}",
                        extracted_path.display()
                    )
                })?;
            } else if metadata.is_file() {
                fs::remove_file(&extracted_path).with_context(|| {
                    format!(
                        "failed to remove old directory: {}",
                        extracted_path.display()
                    )
                })?;
            }
        }
        Err(e) => {
            anyhow::ensure!(
                e.kind() == io::ErrorKind::NotFound,
                "failed to get metadata for path: {}",
                extracted_path.display()
            );
        }
    }
    fs::create_dir(&extracted_path).context("failed to create folder for extraction")?;
    archive::extract(&folder, archive, &extracted_path).context("failed to extract archive")
}
