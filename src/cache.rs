use std::path::PathBuf;
use std::{env, fs};

use anyhow::{Context, Result};
use rouille::url::percent_encoding::{
    USERINFO_ENCODE_SET,
    percent_encode,
};


pub fn cache_dir() -> Result<PathBuf> {
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


pub fn to_cached_path(path: &[u8]) -> String {
    percent_encode(path, USERINFO_ENCODE_SET).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_cached_path() {
        assert_eq!(to_cached_path(b"src/archive"), "src%2Farchive");
        assert_eq!(to_cached_path(b"http://example.com"), "http%3A%2F%2Fexample.com");
        assert_eq!(to_cached_path(b"https://example.com/archive"), "https%3A%2F%2Fexample.com%2Farchive");
        assert_eq!(to_cached_path(br"src\archive"), "src%5Carchive");
        assert_eq!(to_cached_path(br"C:\Temp\archive"), "C%3A%5CTemp%5Carchive");
    }
}
