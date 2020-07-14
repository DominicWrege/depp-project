## Description

A small web app for managing all assignments. [Bootstrap](https://getbootstrap.com/) is used for teh CSS and the [Monaco editor](https://microsoft.github.io/monaco-editor/) to have a better editor then textarea.
[actix-web](https://github.com/actix/actix-web) is used for the http component with [tokio](https://github.com/tokio-rs/tokio) as the async runtime.
[Tera](https://github.com/Keats/tera) is used for rendering the HTMl templates.  
All assignments are stored in a PostgreSQL DB.

## Setup

```
yarn install --modules-folder ./static/node_modules
```

## Build and Run In Debug Mode Mode

```
# For starting the PostgreSQL instance.
docker-compose up -d

cargo run
```

## Config

### API

Web UI to manage all assignments.

| Name              | Type   | Default |
| ----------------- | ------ | ------- |
| DEPP_WEB_PASSWORD | String | secret1 |
| DEPP_WEB_PORT     | uin16  | 5000    |

### RPC

The client config.

| Name                   | Type | Default                |
| ---------------------- | ---- | ---------------------- |
| DEPP_API_LINUX_RPC_URL | URL  | http://127.0.0.1:50051 |
| DEPP_API_MS_RPC_URL    | URL  | http://127.0.0.1:50051 |
