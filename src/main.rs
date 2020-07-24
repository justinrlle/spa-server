mod archive;
mod config;
mod server;

use std::{
    env,
    io::{self},
};

use config::ConfigPath;
use server::Server;

use anyhow::Result;

fn main() -> Result<()> {
    let config_location = env::args_os()
        .nth(1)
        .map(ConfigPath::Provided)
        .unwrap_or(ConfigPath::Default);
    let config = config_location.read()?;

    let server = Server::new(&config.server.folder, &config.proxies)?;
    dbg!(&server.proxies);

    println!(
        "listening on http://{}:{}",
        config.server.host, config.server.port
    );

    let addr = (config.server.host.as_ref(), config.server.port);

    rouille::start_server_with_pool(addr, None, move |request| {
        rouille::log(request, io::stdout(), || server.serve_request(request))
    });
}
