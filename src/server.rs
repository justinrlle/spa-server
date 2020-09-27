use std::{borrow::Cow, collections::HashMap, fs, path::PathBuf, sync::Arc};

use crate::config::ProxyTarget;
use proxy::ProxyConfig;

use anyhow::{Context, Result};
use mime_guess::mime;
use std::time::Duration;

mod proxy;

pub fn log_success(request: &rouille::Request, _response: &rouille::Response, duration: Duration) {
    let method = request.method();
    let path = request.raw_url().split('?').next().unwrap();
    let time = duration.as_millis();
    debug!(
        "{method} {path} - {time}ms",
        method = method,
        path = path,
        time = time
    );
}

pub fn log_error(request: &rouille::Request, duration: Duration) {
    let method = request.method();
    let path = request.raw_url().split('?').next().unwrap();
    let time = duration.as_millis();
    warn!(
        "Handler panicked :{method} {path} - {time}ms",
        method = method,
        path = path,
        time = time
    );
}

pub struct Server {
    pub folder: PathBuf,
    pub http_client: isahc::HttpClient,
    pub proxies: Vec<ProxyConfig>,
}
impl Server {
    pub fn new(folder: PathBuf, proxies: &HashMap<String, ProxyTarget>) -> Result<Arc<Self>> {
        let metadata = fs::metadata(&folder)
            .with_context(|| format!("folder `{}` not found", folder.display()))?;
        anyhow::ensure!(metadata.is_dir(), "`{}` is not a folder", folder.display());
        let http_client = isahc::HttpClient::new().expect("failed to build http client");
        let proxies = proxies
            .iter()
            .map(|(key, val)| ProxyConfig::new(key, val))
            .collect::<Result<_>>()?;
        Ok(Arc::new(Self {
            folder,
            http_client,
            proxies,
        }))
    }
    pub fn serve_request(self: &Arc<Self>, request: &rouille::Request) -> rouille::Response {
        self.clone().inner_serve(request)
    }
    fn inner_serve(&self, request: &rouille::Request) -> rouille::Response {
        for proxy_config in self.proxies.iter() {
            if proxy_config.matches(request) {
                return proxy_config
                    .serve(request, &self.http_client)
                    .unwrap_or_else(error_500);
            }
        }
        self.serve(request)
    }

    fn serve(&self, request: &rouille::Request) -> rouille::Response {
        debug!("serving local file: {}", request.raw_url());
        if wants_html(request) {
            // TODO: nice matching on url to find right html file
            let path = self.folder.join("index.html");
            serve_file(&path, mime::TEXT_HTML_UTF_8)
        } else {
            let path = PathBuf::from(&request.url()[1..]);
            let mime = mime_guess::from_path(&path)
                .first()
                .unwrap_or(mime::APPLICATION_OCTET_STREAM);
            serve_file(&self.folder.join(&path), mime)
        }
    }
}

fn error_500(e: anyhow::Error) -> rouille::Response {
    debug!(
        "raised an internal server error (code 500), caused by: {}",
        e
    );
    rouille::Response::empty_400().with_status_code(500)
}

fn serve_file(file_path: &PathBuf, mime: mime::Mime) -> rouille::Response {
    let mime = Cow::Owned(mime.as_ref().to_owned());
    fs::File::open(file_path)
        .map(|f| rouille::Response::from_file(mime, f))
        .unwrap_or_else(|_| rouille::Response::empty_404())
}

fn wants_html(request: &rouille::Request) -> bool {
    if let Some(accept) = request.header("accept") {
        rouille::input::parse_priority_header(accept)
            .flat_map(|(mime, _)| mime.parse::<mime::Mime>().ok())
            .any(|mime| mime.type_() == mime::TEXT && mime.subtype() == mime::HTML)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_wants_html() {
        use rouille::Request;
        let r = Request::fake_http("GET", "/", vec![("accept".into(), "*/*".into())], vec![]);
        assert!(!wants_html(&r));
        let r = Request::fake_http(
            "GET",
            "/",
            vec![("accept".into(), "text/html".into())],
            vec![],
        );
        assert!(wants_html(&r));
        let r = Request::fake_http(
            "GET",
            "/",
            vec![("accept".into(), "text/html; charset=utf-8".into())],
            vec![],
        );
        assert!(wants_html(&r));
        let r = Request::fake_http(
            "GET",
            "/",
            vec![(
                "accept".into(),
                "text/html; charset=utf-8, text/plain".into(),
            )],
            vec![],
        );
        assert!(wants_html(&r));
        let r = Request::fake_http(
            "GET",
            "/",
            vec![(
                "accept".into(),
                " text/plain, text/html; charset=utf-8, */*".into(),
            )],
            vec![],
        );
        assert!(wants_html(&r));
        // taken from Firefox requesting an html file
        let r = Request::fake_http(
            "GET",
            "/",
            vec![(
                "accept".into(),
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".into(),
            )],
            vec![],
        );
        assert!(wants_html(&r));
        // taken from Firefox requesting a CSS file
        let r = Request::fake_http(
            "GET",
            "/",
            vec![("accept".into(), "text/css,*/*;q=0.1".into())],
            vec![],
        );
        assert!(!wants_html(&r));
    }
}
