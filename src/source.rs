use std::path::PathBuf;

use crate::cache;
use anyhow::Result;

mod archive;
mod http;

#[derive(Clone, Debug)]
pub struct Source<'a> {
    kind: SourceKind,
    app_path: &'a str,
}

#[derive(Clone, Debug)]
enum SourceKind {
    Archive { format: archive::ArchiveFormat },
    Folder,
    Http { format: http::HttpArchive },
}

pub fn detect(app_path: &str) -> Source {
    let kind = if let Some(format) = http::detect(app_path) {
        SourceKind::Http { format }
    } else if let Some(format) = archive::detect(app_path) {
        SourceKind::Archive { format }
    } else {
        SourceKind::Folder
    };
    Source { kind, app_path }
}

impl<'a> Source<'a> {
    pub fn setup(&'a self, cache: &cache::Cache, base_folder: Option<&str>) -> Result<PathBuf> {
        match &self.kind {
            SourceKind::Archive { format } => {
                info!("serving from archive at {}", self.app_path);
                let folder = archive::extract(self.app_path, format, cache)?;
                if let Some(base_folder) = base_folder {
                    let mut folder = folder;
                    folder.push(base_folder);
                    Ok(folder)
                } else {
                    Ok(folder)
                }
            }
            SourceKind::Folder => {
                info!("serving from folder {}", self.app_path);
                Ok(PathBuf::from(self.app_path))
            }
            SourceKind::Http { format } => {
                info!("serving from archive located at {}", self.app_path);
                let folder = http::extract(self.app_path, format, cache)?;
                if let Some(base_folder) = base_folder {
                    let mut folder = folder;
                    folder.push(base_folder);
                    Ok(folder)
                } else {
                    Ok(folder)
                }
            }
        }
    }
}
