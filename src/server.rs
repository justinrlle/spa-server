use std::{borrow::Cow, collections::HashMap, fs, path, sync::Arc};

use crate::config::ProxyTarget;
use proxy::ProxyConfig;

use anyhow::{Context, Result};
use mime_guess::mime;
use std::path::PathBuf;

mod proxy;

pub struct Server {
    pub folder: path::PathBuf,
    pub index: path::PathBuf,
    pub http_client: isahc::HttpClient,
    pub proxies: Vec<ProxyConfig>,
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
        for proxy_config in self.proxies.iter() {
            if request.raw_url().starts_with(&proxy_config.path) {
                return proxy::proxy_request(&self.http_client, request, proxy_config);
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
