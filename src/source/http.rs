use std::path::PathBuf;

use super::archive::{self, ArchiveFormat};

use crate::cache::{Cache, CacheKind};
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

/// This function can only be called for urls that have been constructed by [`detect`](detect).
pub fn extract(app_path: &str, format: &HttpArchive, cache: &Cache) -> Result<PathBuf> {
    anyhow::ensure!(
        format.format.is_tar(),
        "got {:?} archive, only tar archives are supported",
        format.format.kind()
    );
    let cache_path = url_as_cached_path(app_path);
    let _cache_path = cache.path_for_resource(CacheKind::Http, cache_path.as_bytes())?;

    todo!("download and extract archive")
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

fn url_as_cached_path(app_path: &str) -> String {
    let private_url = private_url(app_path);
    assert_ne!(private_url.path(), "/", "url must have a path");
    let last_slash_idx = private_url
        .as_str()
        .rfind('/')
        .expect("all urls have a '/'");
    private_url.as_str()[0..last_slash_idx].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_url() {
        assert_eq!(
            private_url("http://foo:bar@example.com").as_str(),
            "http://example.com/"
        );
        assert_eq!(
            private_url("http://example.com/foo").as_str(),
            "http://example.com/foo"
        );
        assert_eq!(
            private_url("https://example.com/foo").as_str(),
            "https://example.com/foo"
        );
        assert_eq!(
            private_url("http://example.com:8080/foo").as_str(),
            "http://example.com:8080/foo"
        );
        assert_eq!(
            private_url("http://foo:bar@example.com:8080").as_str(),
            "http://example.com:8080/"
        );
    }

    #[test]
    fn test_url_as_cached_path() {
        assert_eq!(
            url_as_cached_path("http://example.com/app.tar.gz"),
            "http://example.com"
        );
        assert_eq!(
            url_as_cached_path("http://example.com/folder/app.tar.gz"),
            "http://example.com/folder"
        );
        assert_eq!(
            url_as_cached_path("http://example.com:8080/app.tar.gz"),
            "http://example.com:8080"
        );
        assert_eq!(
            url_as_cached_path("http://example.com:8080/folder/app.tar.gz"),
            "http://example.com:8080/folder"
        );
        assert_eq!(
            url_as_cached_path("http://foo:bar@example.com/app.tar.gz"),
            "http://example.com"
        );
        assert_eq!(
            url_as_cached_path("http://foo:bar@example.com/folder/app.tar.gz"),
            "http://example.com/folder"
        );
    }
    #[test]
    #[should_panic]
    fn test_url_as_cached_path_expects_path() {
        url_as_cached_path("http://example.com");
    }
}
