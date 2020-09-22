use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use rouille::url::percent_encoding::{percent_encode, USERINFO_ENCODE_SET};

#[derive(Debug)]
pub struct Cache {
    cache_folder: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheKind {
    Archive,
    Http,
}

impl CacheKind {
    fn as_folder(&self) -> &'static str {
        match self {
            CacheKind::Archive => "archive",
            CacheKind::Http => "http",
        }
    }
}

impl Cache {
    pub fn init() -> Result<Self> {
        let temp_dir = env::temp_dir();
        let project_dirs = directories::ProjectDirs::from("beer", "justinrlle", "Spa-Server");
        let cache_folder = project_dirs
            .map(|p| p.cache_dir().to_owned())
            .unwrap_or_else(|| temp_dir.clone());
        debug!("cache folder: {}", cache_folder.display());
        fs::create_dir_all(&cache_folder)
            .with_context(|| format!("failed to create cache path: {}", cache_folder.display()))?;
        Ok(Self { cache_folder })
    }

    pub fn path_for_resource(&self, kind: CacheKind, resource: &[u8]) -> Result<PathBuf> {
        let path = self.cache_folder.join(kind.as_folder());
        ensure_path_exists(&path)?;
        let path = path.join(to_cached_path(resource));
        ensure_path_exists(&path)?;
        Ok(path)
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn init_with_custom_path_for_test(cache_folder: PathBuf) -> Self {
        Self { cache_folder }
    }
}

fn to_cached_path(path: &[u8]) -> String {
    percent_encode(path, USERINFO_ENCODE_SET).to_string()
}

fn ensure_path_exists(path: &Path) -> Result<()> {
    match fs::metadata(path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                fs::remove_dir_all(path).with_context(|| {
                    format!("failed to remove old directory: {}", path.display())
                })?;
            } else if metadata.is_file() {
                fs::remove_file(path).with_context(|| {
                    format!("failed to remove old directory: {}", path.display())
                })?;
            }
        }
        Err(e) => {
            anyhow::ensure!(
                e.kind() == io::ErrorKind::NotFound,
                "failed to get metadata for path: {}",
                path.display()
            );
        }
    }
    fs::create_dir(path).with_context(|| format!("failed to create folder: {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_cached_path() {
        assert_eq!(to_cached_path(b"src/archive"), "src%2Farchive");
        assert_eq!(
            to_cached_path(b"http://example.com"),
            "http%3A%2F%2Fexample.com"
        );
        assert_eq!(
            to_cached_path(b"https://example.com/archive"),
            "https%3A%2F%2Fexample.com%2Farchive"
        );
        assert_eq!(to_cached_path(br"src\archive"), "src%5Carchive");
        assert_eq!(to_cached_path(br"C:\Temp\archive"), "C%3A%5CTemp%5Carchive");
    }
}
