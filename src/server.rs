use std::{borrow::Cow, collections::HashMap, fs, io::Read as _, path, sync::Arc};

use crate::config::ProxyTarget;

use anyhow::{Context, Result};
use isahc::http;
use mime_guess::mime;
use rouille::url;
use std::path::PathBuf;

pub struct Server {
    pub folder: path::PathBuf,
    pub index: path::PathBuf,
    pub http_client: isahc::HttpClient,
    pub proxies: Vec<ProxyConfig>,
}

#[derive(Debug)]
pub struct ProxyConfig {
    pub path: String,
    pub target: String,
    pub path_rewrite: Option<(String, String)>,
    pub secure: bool,
    pub headers: HashMap<String, String>,
}

impl ProxyConfig {
    pub fn new(path: &str, proxy: &ProxyTarget) -> Result<Self> {
        anyhow::ensure!(path.starts_with('/'), "path `{}` is not a valid path", path);
        let mut path = path.to_owned();
        if !path.ends_with('/') {
            path += "/";
        }
        url::Url::parse(&proxy.target).context("invalid target")?;
        let target = proxy.target.clone();
        let path_rewrite = proxy.path_rewrite.clone();
        let secure = proxy.secure;
        let headers = proxy.headers.clone();
        Ok(Self {
            path,
            target,
            path_rewrite,
            secure,
            headers,
        })
    }
}

impl Server {
    pub fn new(folder: PathBuf, proxies: &HashMap<String, ProxyTarget>) -> Result<Arc<Self>> {
        let metadata = fs::metadata(&folder)
            .with_context(|| format!("folder `{}` not found", folder.display()))?;
        anyhow::ensure!(metadata.is_dir(), "`{}` is not a folder", folder.display());
        let index = folder.join("index.html");
        let metadata = fs::metadata(&index)
            .with_context(|| format!("no index.html found in `{}`", folder.display()))?;
        anyhow::ensure!(metadata.is_file(), "index.html is not a file");
        let http_client = isahc::HttpClient::new().expect("failed to build http client");
        let proxies = proxies
            .iter()
            .map(|(key, val)| ProxyConfig::new(key, val))
            .collect::<Result<_>>()?;
        Ok(Arc::new(Self {
            folder,
            index,
            http_client,
            proxies,
        }))
    }
    pub fn serve_request(self: &Arc<Self>, request: &rouille::Request) -> rouille::Response {
        self.clone().inner_serve(request).expect("server error")
    }
    fn inner_serve(&self, request: &rouille::Request) -> Result<rouille::Response> {
        for proxy in self.proxies.iter() {
            if request.raw_url().starts_with(&proxy.path) {
                let req = rouille_to_isahc(request, proxy.target.as_ref());
                let res = self.http_client.send(req).with_context(|| {
                    format!("failed to forward request to proxy at {}", proxy.target)
                })?;
                return Ok(isahc_to_rouille(res));
            }
        }

        if wants_html(request) {
            return Ok(serve_file(&self.index, mime::TEXT_HTML_UTF_8));
        }
        let path = path::PathBuf::from(&request.url()[1..]);
        Ok(self.try_serve(&path))
    }

    fn try_serve(&self, path: &path::PathBuf) -> rouille::Response {
        let mime = mime_guess::from_path(path)
            .first()
            .unwrap_or(mime::APPLICATION_OCTET_STREAM);
        serve_file(&self.folder.join(path), mime)
    }
}

fn serve_file(file_path: &path::PathBuf, mime: mime::Mime) -> rouille::Response {
    let mime = Cow::Owned(mime.as_ref().to_owned());
    fs::File::open(file_path)
        .map(|f| rouille::Response::from_file(mime, f))
        .unwrap_or_else(|_| rouille::Response::empty_404())
}

fn wants_html(request: &rouille::Request) -> bool {
    if let Some(accept) = request.header("accept") {
        accept
            .split(',')
            .flat_map(|mime| mime.parse::<mime::Mime>().ok())
            .any(|mime| mime.type_() == mime::TEXT && mime.subtype() == mime::HTML)
    } else {
        false
    }
}

// must set "connection" header to "close"
fn rouille_to_isahc(req: &rouille::Request, url: &str) -> http::Request<isahc::Body> {
    let builder = http::Request::builder()
        .method(req.method())
        .uri(url.to_owned() + req.raw_url());
    let mut builder = req
        .headers()
        .fold(builder, |builder, (key, value)| builder.header(key, value));
    {
        let headers = builder.headers_mut().expect("no header map");
        headers.insert("connection", http::HeaderValue::from_static("close"));
    }
    let mut data = req.data().expect("no data found");
    let mut buffer = Vec::new();
    let size = data
        .read_to_end(&mut buffer)
        .expect("failed to read from incoming request");
    if size == 0 {
        builder
            .body(isahc::Body::empty())
            .expect("failed to build request")
    } else {
        builder
            .body(isahc::Body::from_bytes(&buffer))
            .expect("failed to build request")
    }
}

fn isahc_to_rouille(res: http::Response<isahc::Body>) -> rouille::Response {
    let status_code = res.status().as_u16();
    let headers = res
        .headers()
        .iter()
        .map(|(key, value)| {
            let key = Cow::Owned(key.as_str().to_owned());
            let value = Cow::Owned(
                value
                    .to_str()
                    .expect("header value is not valid utf-8")
                    .to_owned(),
            );
            (key, value)
        })
        .collect::<Vec<_>>();
    let body = res.into_body();
    let data = if body.is_empty() {
        rouille::ResponseBody::empty()
    } else {
        rouille::ResponseBody::from_reader(body)
    };
    rouille::Response {
        status_code,
        headers,
        data,
        upgrade: None,
    }
}
