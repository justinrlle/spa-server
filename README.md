# `spa-server`

A simple cli tool to serve a folder containing a SPA application, with basic proxy capabilities.

## Features

- serve from a folder using cli args or a config file
- respond to `html` requests with the root `index.html`
- serve from a tar archive (the `tar` executable must be present)
- serve from an url pointing to a tar archive (_soon™_)
- proxy some calls to other apps (à la [webpack dev-server proxy][devserverproxy], but with less features)
- use `~` and environnement variables in application path

## Example

Assuming an angular v8 app called `project-name`, and built with `ng build`.

In one line:
```shell script
$ spa-server -s dist/project-name
# will serve on http://localhost:4242
```

### Using a config file

If you want to have proxies, or customize the host and the port, you must use a config file. Given one called `Spa-project-name.toml`:
```toml
[server]
port = 4200
folder = "~/git/project-name/dist/project-name" # support for `~` in paths

[proxies]
"/api" = { target = "http://localhost:8080" }
# example: http://localhost:4200/api/hello will be forwarded to http://localhost:8080/api/hello
```
Just run `spa-server Spa-project-name.toml` and you're ready to go!

For a complete documentation about the config file, your best bet right now is the [documentation of the `Config` `struct`](./src/config.rs).

## Installation

### Compile from sources

First, [Rust](https://www.rust-lang.org) must be installed. You can then install this binary from git:
```shell
$ cargo install --force --git https://github.com/justinrlle/spa-server
```

## TODO (and current caveats)

### TODO soon
- serve from url
- figure out how to use secrets in application path
  - plain old environnement variables?
  - `dotenv` file loading?
  -  use of `secret-tool`, `gnome-keychain`, Keychain Access on macOS?

### long term TODOs
- other protocols
  - s3 (`aws-cli` or `rusoto`?)
  - ftp
  - ssh?
  - git?
- windows support for archives
- better testing for archives
- feature parity with [webpack dev-server proxy][devserverproxy]
- cli option to define simple proxies (possible syntax: `spa-server --forward /api=http://localhost:8080`)

[devserverproxy]: https://webpack.js.org/configuration/dev-server/#devserverproxy
