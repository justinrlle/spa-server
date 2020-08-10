use crate::config::ProxyTarget;
use anyhow::{Context, Result};
use isahc::http;
use rouille::url;
use std::{borrow::Cow, collections::HashMap, io::Read as _};

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

pub fn proxy_request(
    http_client: &isahc::HttpClient,
    request: &rouille::Request,
    proxy_config: &ProxyConfig,
) -> Result<rouille::Response> {
    let req = rouille_to_isahc(request, proxy_config.target.as_ref());
    let res = http_client.send(req);
    if let Err(e) = &res {
        error!("failed to proxy request to {}: {}", proxy_config.target, e);
    }
    let res = res?;
    Ok(isahc_to_rouille(res))
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
