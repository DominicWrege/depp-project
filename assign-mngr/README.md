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

## Build And Publish The Docker Image

```
sudo docker build -t web -f ../Docker-Files/Dockerfile-Assign-Mngr-Release ..
sudo docker tag web dominicwrege/depp-project-web
sudo docker push dominicwrege/depp-project-web
```
