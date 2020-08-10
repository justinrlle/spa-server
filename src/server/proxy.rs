use crate::config::ProxyTarget;
use anyhow::{Context, Result};
use isahc::{http, HttpClient};
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
        rouille::url::Url::parse(&proxy.target).context("invalid target")?;
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

    pub fn matches(&self, request: &rouille::Request) -> bool {
        request.raw_url().starts_with(&self.path)
    }

    pub fn serve(
        &self,
        request: &rouille::Request,
        http_client: &HttpClient,
    ) -> Result<rouille::Response> {
        info!("proxying request at {} to {}", request.url(), self.target);
        let req = rouille_to_http(request, self.target.as_ref());
        let res = http_client.send(req);
        if let Err(e) = &res {
            warn!("failed to proxy request to {}: {}", self.target, e);
        }
        let res = res?;
        Ok(http_to_rouille(res))
    }
}

fn rouille_to_http(req: &rouille::Request, url: &str) -> http::Request<isahc::Body> {
    let builder = http::Request::builder()
        .method(req.method())
        .uri(url.to_owned() + req.raw_url());
    let builder = req
        .headers()
        .fold(builder, |builder, (key, value)| builder.header(key, value));
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

fn http_to_rouille(res: http::Response<isahc::Body>) -> rouille::Response {
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
