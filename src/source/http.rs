use std::{fs, io, path::PathBuf};

use super::archive::{self, ArchiveFormat};

use crate::cache::{Cache, CacheKind};
use anyhow::{Context, Result};
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
    let filename = url_filename(app_path);
    let download_path = cache
        .resource(
            CacheKind::Http,
            cache_path_for_download(app_path).as_bytes(),
        )
        .context("failed to setup folder for downloading archive")?
        .join(&filename);
    trace!("downloading to: {}", download_path.display());

    let mut download_file = fs::File::create(&download_path).context("failed to download file")?;

    let mut body = isahc::get(app_path)
        .context("failed to download file")?
        .into_body();
    io::copy(&mut body, &mut download_file).context("failed to download file")?;

    let extract_path = cache
        .resource(
            CacheKind::Archive,
            cache_path_for_extraction(app_path, &format.format).as_bytes(),
        )
        .context("failed to setup folder for downloading archive")?
        .join(&filename);

    trace!("extracting to: {}", extract_path.display());

    archive::extract_archive_to(&download_path, &format.format, &extract_path)
        .context("failed to extract archive")?;

    Ok(extract_path)
}

fn private_url(url: &str) -> url::Url {
    let url = url::Url::parse(url).expect("the provided url is not a valid one");
    assert_ne!(url.path(), "/", "url must have a path");
    url::Url::parse(&format!(
        "{origin}{path}",
        origin = url.origin().ascii_serialization(),
        path = url.path(),
    ))
    .expect("failed to create private url")
}

fn cache_path_for_download(app_path: &str) -> String {
    let private_url = private_url(app_path);
    let last_slash_idx = private_url
        .as_str()
        .rfind('/')
        .expect("all urls have a '/'");
    private_url.as_str()[0..last_slash_idx].to_owned()
}

fn cache_path_for_extraction(app_path: &str, format: &ArchiveFormat) -> String {
    let private_url = private_url(app_path);
    format.strip_self(private_url.as_str()).to_owned()
}

fn url_filename(app_path: &str) -> String {
    let private_url = private_url(app_path);
    let last_slash_idx = private_url.path().rfind('/').expect("all urls have a '/'");
    private_url.path()[last_slash_idx + 1..].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_url() {
        assert_eq!(
            private_url("http://foo:bar@example.com/foo").as_str(),
            "http://example.com/foo"
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
            private_url("http://foo:bar@example.com:8080/foo").as_str(),
            "http://example.com:8080/foo"
        );
    }

    #[test]
    fn test_url_as_cached_path() {
        assert_eq!(
            cache_path_for_download("http://example.com/app.tar.gz"),
            "http://example.com"
        );
        assert_eq!(
            cache_path_for_download("http://example.com/folder/app.tar.gz"),
            "http://example.com/folder"
        );
        assert_eq!(
            cache_path_for_download("http://example.com:8080/app.tar.gz"),
            "http://example.com:8080"
        );
        assert_eq!(
            cache_path_for_download("http://example.com:8080/folder/app.tar.gz"),
            "http://example.com:8080/folder"
        );
        assert_eq!(
            cache_path_for_download("http://foo:bar@example.com/app.tar.gz"),
            "http://example.com"
        );
        assert_eq!(
            cache_path_for_download("http://foo:bar@example.com/folder/app.tar.gz"),
            "http://example.com/folder"
        );
    }
    #[test]
    #[should_panic]
    fn test_url_as_cached_path_expects_path() {
        cache_path_for_download("http://example.com");
    }
}
