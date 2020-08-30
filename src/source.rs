use std::path::{Path, PathBuf};

use anyhow::Result;
use rouille::url;

mod archive;

#[derive(Copy, Clone, Debug)]
pub enum SourceKind {
    Archive {
        format: archive::ArchiveFormat,
    },
    Folder,
    Url {
        archive_format: archive::ArchiveFormat,
    },
}

pub fn detect(app_path: &str) -> SourceKind {
    if let Some(format) = archive::detect(app_path) {
        return SourceKind::Archive { format };
    };
    if let Some(archive_format) = url::Url::parse(app_path)
        .ok()
        .and_then(|url| archive::detect(url.path()))
    {
        return SourceKind::Url { archive_format };
    }
    SourceKind::Folder
}

impl SourceKind {
    pub fn setup(
        &self,
        app_path: &str,
        cache_folder: &Path,
        base_folder: Option<&str>,
    ) -> Result<PathBuf> {
        match self {
            SourceKind::Archive { format } => {
                anyhow::ensure!(
                    format.is_tar(),
                    "got {:?} archive, only tar archives are supported",
                    format.kind()
                );
                info!("serving from archive at {}", app_path);
                let folder = archive::extract(app_path, format, cache_folder)?;
                if let Some(base_folder) = base_folder {
                    let mut folder = folder;
                    folder.push(base_folder);
                    Ok(folder)
                } else {
                    Ok(folder)
                }
            }
            SourceKind::Folder => {
                info!("serving from folder {}", app_path);
                Ok(PathBuf::from(app_path))
            }
            SourceKind::Url {
                archive_format: _archive_format,
            } => todo!("add url source"),
        }
    }
}
