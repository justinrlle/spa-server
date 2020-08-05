mod archive;
mod config;
mod server;

use std::{
    env, fs,
    io::{self},
};

use config::ConfigPath;
use server::Server;

use anyhow::{Context, Result};
use std::path::PathBuf;

fn main() -> Result<()> {
    let config_location = env::args_os()
        .nth(1)
        .map(ConfigPath::Provided)
        .unwrap_or(ConfigPath::Default);
    let config = config_location.read()?;

    let folder = shellexpand::tilde(&config.server.folder);
    let archive = archive::detect(&folder);

    let folder = if let Some(archive) = archive {
        anyhow::ensure!(
            archive.is_tar(),
            "got {:?} archive, only tar archives are supported",
            archive.kind
        );
        let temp_dir = env::temp_dir();
        let project_dirs = directories::ProjectDirs::from("beer", "justinrlle", "Spa-Server");
        let cache_folder = project_dirs
            .map(|p| p.cache_dir().to_owned())
            .unwrap_or_else(|| temp_dir.clone());
        dbg!(cache_folder.display());
        fs::create_dir_all(&cache_folder)
            .with_context(|| format!("failed to create cache path: {}", cache_folder.display()))?;
        let extracted_path = archive::extract_path(&folder, archive, &cache_folder)
            .context("failed to deduce extracted path for archive")?;
        dbg!(&extracted_path);
        match fs::metadata(&extracted_path) {
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
        archive::extract(&folder, archive, &extracted_path).context("failed to extract archive")?;
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

    dbg!(&folder);

    let server = Server::new(folder, &config.proxies)?;
    dbg!(&server.proxies);

    println!(
        "listening on http://{}:{}",
        config.server.host, config.server.port
    );

    let addr = (config.server.host.as_ref(), config.server.port);

    rouille::start_server_with_pool(addr, None, move |request| {
        rouille::log(request, io::stdout(), || server.serve_request(request))
    });
}
