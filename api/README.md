## Description

This is the REST API which uses `JSON`. It communicates via RPC to the RPC servers (Windows | Linux) by retrieving all assignments from the PostgreSQL Database.
[actix-web](https://github.com/actix/actix-web) is used for the http component with [tokio](https://github.com/tokio-rs/tokio) as the async runtime.

## Build and Run In Debug Mode

```
# For starting the PostgreSQL instance.
docker-compose -f ../assign-mngr/docker-compose.yml up

cargo run
```

**Endpoint**  
Local:
[http://localhost:6000/api](http://localhost:6000/api)  
Remote:
[https://auth.inf.fh-dortmund.de:2443/api](https://auth.inf.fh-dortmund.de:2443/api)

## Config

The REST API uses basic access authentication configured via environment variables.

### Basic Auth

| Name              | Type   | Default  |
| ----------------- | ------ | -------- |
| DEPP_API_USERNAME | String | user     |
| DEPP_API_PASSWORD | String | wasd4221 |

### RPC

The client config.

| Name                   | Type | Default                |
| ---------------------- | ---- | ---------------------- |
| DEPP_API_LINUX_RPC_URL | URL  | http://127.0.0.1:50051 |
| DEPP_API_MS_RPC_URL    | URL  | http://127.0.0.1:50051 |

## OpenAPI REST Documentation

```
# start
docker-compose -f ./openapi/docker-compose.yml up
# stop
docker-compose -f ./openapi/docker-compose.yml down
```

## Build And Publish The Docker Image

```
sudo docker build -t api -f ../Docker-Files/Dockerfile-API ..
sudo docker tag api dominicwrege/depp-project-api
sudo docker push dominicwrege/depp-project-api
```
