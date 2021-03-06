# Depp Project

## Overview

This project contains of 5 crates (3 Main Crates and 2 helper lib)

- **api**: The JSON REST API with and openapi documentation inside (main)
- **assign-mngr**: The web UI (Assignments Manager) for managing all assignments (main)
- **testing**: For running all scripts inside a Docker (main)
- **db_lib**: Shared code for connection to PostgreSQL (helper lib)
- **grpc_lib**: All models written in protobuf (helper lib)

## Usage

### Run

The testing and api server are booth prebuilt available on dockerhub.
So now it should take a couple seconds.

```
$ docker-compose up
```

**API Endpoint**  
Local:
[http://localhost:6000/api](http://localhost:6000/api) (See Swagger UI for more information.)  
Remote:
[https://auth.inf.fh-dortmund.de:2443/api](https://auth.inf.fh-dortmund.de:2443/api)

**Assignments Manager**  
Local:
[http://localhost:5000/manage](http://localhost:5000/manage)  
Remote:
[https://auth.inf.fh-dortmund.de:2443/manage](https://auth.inf.fh-dortmund.de:2443/manage)

**REST API Documentation**
Local:
[http://localhost:4000](http://localhost:4000)

## Config

### API

The REST API uses basic access authentication configured via environment variables.

#### Basic Auth

| Name              | Type   | Default  |
| ----------------- | ------ | -------- |
| DEPP_API_USERNAME | String | user     |
| DEPP_API_PASSWORD | String | wasd4221 |

#### RPC

The client config.

| Name                   | Type | Default                |
| ---------------------- | ---- | ---------------------- |
| DEPP_API_LINUX_RPC_URL | URL  | http://127.0.0.1:50051 |
| DEPP_API_MS_RPC_URL    | URL  | http://127.0.0.1:50051 |

### Testing

| Name                   | Type                | Default                                                                                          |
| ---------------------- | ------------------- | ------------------------------------------------------------------------------------------------ |
| DEPP_TEST_PORT         | uint16              | 50051                                                                                            |
| DEPP_TEST_MAX_CURR     | uint8               | Linux: 10, Windows: 5                                                                            |
| DEPP_TEST_DOCKER_IMAGE | String              | Linux: `dominicwrege/depp-project-ubuntu:latest`, Windows: `mcr.microsoft.com/powershell:latest` |
| DEPP_TEST_TIMEOUT      | uint64 format: secs | Linux: 120, Windows: 180                                                                         |

### Assignment Manager

| Name              | Type   | Default |
| ----------------- | ------ | ------- |
| DEPP_WEB_PASSWORD | String | secret1 |
| DEPP_WEB_PORT     | uin16  | 5000    |

#### PostgreSQL

Booth API and Assignment Manager using this.

| Name                    | Type   | Default     |
| ----------------------- | ------ | ----------- |
| POSTGRES_USER           | String | johncena    |
| POSTGRES_PASSWORD       | String | secret1     |
| POSTGRES_DB             | String | assignments |
| POSTGRES_PORT           | uint16 | 5432        |
| POSTGRES_HOST           | String | localhost   |
| POSTGRES_MAX_CONNECTION | uint16 | 16          |

### Example

For testing purpose you can use the default config `./example.env`:

```
# db
POSTGRES_USER=johncena
POSTGRES_PASSWORD=supersecret
POSTGRES_DB=assignments
POSTGRES_HOST=db

# basic auth
DEPP_API_USERNAME=tester
DEPP_API_PASSWORD=whatever
# RPC config
DEPP_API_LINUX_RPC_URL=http://testing:50051

# web ui manage thing
DEPP_WEB_PASSWORD=mypassword
DEPP_WEB_PORT=5000

```

## Description

Please take a look inside the examples folder.All assignments are stored in PostgreSQL.
The Assignments Manager allows you to create/edit assignments.
I included some examples scripts folder in the `.testing/example-scripts` folder.

### How Scripts Are Tested

Each Script will run inside a docker container for max duration of 120 secs on Linux and on Windows 180 secs and the memory is limited to 200-320MB. First the solution script will run. After that the provided script will run. At the end both stdouts and writing files are 1:1 compared

### Currently Supported Script Types

- Python3
- PowerShell\*
- Shell
- Bash
- Batch\*
- Sed
- Awk

\*Windows only because I'm using windows containers for this.

## Code Documentation

For how to use the API from outside everything is written in `./api/openapi/doc.openapi.yml` file.
To create documentation for the Rust code:

```
# --open is optional and be sure that a root at this repo
cargo doc --no-deps --open
```

## How To Create and Publish Images to Dockerhub

Build and publish the Assignment Manager image (assign-mngr)

```
# from the root folder of this repo
sudo docker build -t web -f Docker-Files/Dockerfile-Assign-Mngr-Release  .
sudo docker tag web dominicwrege/depp-project-web:latest
sudo docker push dominicwrege/depp-project-web
```

Build and publish the Rest API image:

```
# from the root folder of this repo
sudo docker build -t api -f Docker-Files/Dockerfile-API .
sudo docker tag api dominicwrege/depp-project-api:latest
sudo docker push dominicwrege/depp-project-api
```

Build and publish the testing image:

```
# from the root folder of this repo
sudo docker build -t testing -f Docker-Files/Dockerfile-Testing .
sudo docker tag testing dominicwrege/depp-project-testing:latest
sudo docker push dominicwrege/depp-project-testing
```

## Backup And Restore Database

**_Note_**: Make sure you have the matching postgresql-client installed.
See how to [install postgresql 12 on Ubuntu.](https://computingforgeeks.com/install-postgresql-12-on-ubuntu/)

Backup:

```
pg_dump postgres://USER:"PASSWORD"@127.0.0.1:5432/assignments > db-backup
```

Restore:

```
psql postgres://USER:"PASSWORD"@127.0.0.1:5432/assignments < db-backup
```

Read the PostgrSQL [documentation](https://www.postgresql.org/docs/current/backup-dump.html).
