use std::path::{Path, PathBuf};

use super::archive::{self, ArchiveFormat};

use anyhow::Result;
use rouille::url;

#[derive(Clone, Debug)]
pub struct HttpArchive {
    format: ArchiveFormat,
}

pub fn detect(app_path: &str) -> Option<HttpArchive> {
    url::Url::parse(app_path)
        .ok()
        .filter(|url| url.scheme() == "http" || url.scheme() == "https")
        .and_then(|url| archive::detect(url.path()))
        .map(|format| HttpArchive { format })
}

fn private_url(url: &str) -> url::Url {
    let url = url::Url::parse(url).expect("the provided url is not a valid one");
    url::Url::parse(&format!(
        "{origin}{path}",
        origin = url.origin().ascii_serialization(),
        path = url.path(),
    ))
    .expect("failed to create private url")
}

/// This function can only be called for urls that have been constructed by [`detect`](detect).
pub fn extract(app_path: &str, format: &HttpArchive, _cache_folder: &Path) -> Result<PathBuf> {
    anyhow::ensure!(
        format.format.is_tar(),
        "got {:?} archive, only tar archives are supported",
        format.format.kind()
    );
    let private_url = private_url(app_path);
    let last_slash_idx = private_url.as_str().rfind('/').expect("no '/' in url");
    let _parent_path = &private_url.as_str()[0..last_slash_idx];

    todo!("download and extract archive")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_url() {
        assert_eq!(
            private_url("http://foo:bar@example.com").to_string(),
            "http://example.com/"
        );
        assert_eq!(
            private_url("http://example.com/foo").to_string(),
            "http://example.com/foo"
        );
        assert_eq!(
            private_url("http://example.com:8080/foo").to_string(),
            "http://example.com:8080/foo"
        );
    }
}
