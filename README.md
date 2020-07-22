# `spa-server`

A simple cli tool to serve a folder containing a SPA application. There is proxy capabilities, so
one can imitate the [webpack dev-server proxy](https://webpack.js.org/configuration/dev-server/#devserverproxy).

## Example config

```toml
# serving an angular app built with @angular/cli@8
[server]
port = 4200
folder = "~/git/project-name/dist/project-name" # support for `~` in paths

[proxies]
"/api" = { target = "http://localhost:8080" }
```
