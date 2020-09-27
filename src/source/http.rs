use std::{fs, io, path::PathBuf};

use super::archive::{self, ArchiveFormat};

use crate::cache::{Cache, CacheKind};
use anyhow::{Context, Result};
use isahc::http;
use rouille::url::Url;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Debug)]
pub struct HttpArchive {
    format: ArchiveFormat,
}

pub fn detect(app_path: &str) -> Option<HttpArchive> {
    Url::parse(app_path)
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
    let (private_url, req) = url_and_request(app_path);
    let filename = url_filename(&private_url);
    trace!("filename: {}", filename);
    let download_path = cache
        .resource(
            CacheKind::Http,
            &[cache_path_for_download(&private_url).as_bytes()],
        )
        .context("failed to setup folder for downloading archive")?
        .join(&filename);
    trace!("downloading to: {}", download_path.display());

    let mut download_file = fs::File::create(&download_path).context("failed to download file")?;

    let response = isahc::send(req).context("failed to download file")?;
    trace!(
        "response: status={} version={:?}",
        response.status(),
        response.version()
    );
    trace!("response headers: {:#?}", response.headers());
    anyhow::ensure!(
        response.status().is_success(),
        "download failed: got status {}",
        response.status()
    );
    let mut body = response.into_body();
    io::copy(&mut body, &mut download_file).context("failed to download file")?;

    let extract_path = cache
        .resource(
            CacheKind::Archive,
            &[
                cache_path_for_extraction(&private_url, &format.format).as_bytes(),
                format.format.strip_self(&filename).as_bytes(),
            ],
        )
        .context("failed to setup folder for downloading archive")?;

    trace!("extracting to: {}", extract_path.display());

    archive::extract_archive_to(&download_path, &format.format, &extract_path)
        .context("failed to extract archive")?;

    Ok(extract_path)
}

fn cache_path_for_download(private_url: &Url) -> String {
    let last_slash_idx = private_url
        .as_str()
        .rfind('/')
        .expect("all urls have a '/'");
    private_url.as_str()[0..last_slash_idx].to_owned()
}

fn cache_path_for_extraction(private_url: &Url, format: &ArchiveFormat) -> String {
    format.strip_self(private_url.as_str()).to_owned()
}

fn url_filename(private_url: &Url) -> String {
    let last_slash_idx = private_url.path().rfind('/').expect("all urls have a '/'");
    private_url.path()[last_slash_idx + 1..].to_owned()
}

fn url_and_request(app_path: &str) -> (Url, isahc::http::Request<()>) {
    let url = Url::parse(app_path).expect("the provided url is not a valid one");
    assert_ne!(url.path(), "/", "url must have a path");
    let private_url = Url::parse(&format!(
        "{origin}{path}",
        origin = url.origin().ascii_serialization(),
        path = url.path(),
    ))
    .expect("failed to create private url");
    let mut builder = http::Request::get(private_url.as_str())
        .header(http::header::USER_AGENT, USER_AGENT)
        .header(http::header::ACCEPT, "*/*");
    if let Some(password) = url.password() {
        if !url.username().is_empty() {
            let basic_token = base64::encode(&format!("{}:{}", url.username(), password));
            builder = builder.header(
                http::header::AUTHORIZATION,
                format!("Basic {}", basic_token),
            );
        }
    };
    (
        private_url,
        builder.body(()).expect("failed to build request"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_url_and_request() {
        let (url, req) = url_and_request("http://foo:bar@example.com/foo");
        assert_eq!(url.as_str(), "http://example.com/foo");
        assert_eq!(req.uri(), "http://example.com/foo");
        assert_eq!(
            req.headers().get(http::header::AUTHORIZATION).unwrap(),
            "Basic Zm9vOmJhcg=="
        );

        let (url, req) = url_and_request("http://example.com/foo");
        assert_eq!(url.as_str(), "http://example.com/foo");
        assert_eq!(req.uri(), "http://example.com/foo");
        assert_eq!(req.headers().get(http::header::AUTHORIZATION), None);

        let (url, req) = url_and_request("https://example.com/foo");
        assert_eq!(url.as_str(), "https://example.com/foo");
        assert_eq!(req.uri(), "https://example.com/foo");
        assert_eq!(req.headers().get(http::header::AUTHORIZATION), None);

        let (url, req) = url_and_request("http://example.com:8080/foo");
        assert_eq!(url.as_str(), "http://example.com:8080/foo");
        assert_eq!(req.uri(), "http://example.com:8080/foo");
        assert_eq!(req.headers().get(http::header::AUTHORIZATION), None);

        let (url, req) = url_and_request("http://foo:bar@example.com:8080/foo");
        assert_eq!(url.as_str(), "http://example.com:8080/foo");
        assert_eq!(req.uri(), "http://example.com:8080/foo");
        assert_eq!(
            req.headers().get(http::header::AUTHORIZATION).unwrap(),
            "Basic Zm9vOmJhcg=="
        );
    }

    #[test]
    fn test_cache_path_for_download() {
        assert_eq!(
            cache_path_for_download(&Url::parse("http://example.com/app.tar.gz").unwrap()),
            "http://example.com"
        );
        assert_eq!(
            cache_path_for_download(&Url::parse("http://example.com/folder/app.tar.gz").unwrap()),
            "http://example.com/folder"
        );
        assert_eq!(
            cache_path_for_download(&Url::parse("http://example.com:8080/app.tar.gz").unwrap()),
            "http://example.com:8080"
        );
        assert_eq!(
            cache_path_for_download(
                &Url::parse("http://example.com:8080/folder/app.tar.gz").unwrap()
            ),
            "http://example.com:8080/folder"
        );
    }
    #[test]
    #[should_panic]
    fn test_url_and_request_expects_path() {
        url_and_request("http://example.com");
    }

    // TODO: more tests
}
