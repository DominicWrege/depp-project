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
# Don't forget to install the node_modules first
# For starting the PostgreSQL instance.
docker-compose up -d
# Run
cargo run
```

Open [localhost:5000/manage](http://localhost:5000/manage) in your browser.

## Config

Set the server port and the password for the admin user.

| Name              | Type   | Default |
| ----------------- | ------ | ------- |
| DEPP_WEB_PASSWORD | String | secret1 |
| DEPP_WEB_PORT     | uin16  | 5000    |

## Build And Publish The Docker Image

```
sudo docker build -t web -f ../Docker-Files/Dockerfile-Assign-Mngr-Release ..
sudo docker tag web dominicwrege/depp-project-web:latest
sudo docker push dominicwrege/depp-project-web
```
