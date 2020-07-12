# Usage

## Run

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

**Swagger UI**  
Local:
[http://localhost:4000](http://localhost:4000)

# Config

## API

The REST API uses HTTP basic auth passed via environment variable.

### Basic Auth

| Name              | Type   | Default  |
| ----------------- | ------ | -------- |
| DEPP_API_USERNAME | String | user     |
| DEPP_API_PASSWORD | String | wasd4221 |

### RPC

| Name                   | Type | Default                |
| ---------------------- | ---- | ---------------------- |
| DEPP_API_LINUX_RPC_URL | URL  | http://127.0.0.1:50051 |
| DEPP_API_MS_RPC_URL    | URL  | http://127.0.0.1:50051 |

## Testing

| Name                   | Type                | Default                                                                                          |
| ---------------------- | ------------------- | ------------------------------------------------------------------------------------------------ |
| DEPP_TEST_PORT         | uint16              | 50051                                                                                            |
| DEPP_TEST_MAX_CURR     | uint8               | Linux: 8, Windows: 4                                                                             |
| DEPP_TEST_DOCKER_IMAGE | String              | Linux: `dominicwrege/depp-project-ubuntu:latest`, Windows: `mcr.microsoft.com/powershell:latest` |
| DEPP_TEST_TIMEOUT      | uint64 format: secs | Linux: 120, Windows: 180                                                                         |

## Assignment Manager

| Name              | Type   | Default |
| ----------------- | ------ | ------- |
| DEPP_WEB_PASSWORD | String | secret1 |
| DEPP_WEB_PORT     | uin16  | 5000    |

### PostgreSQL

Booth API and Assignment Manager using this.

| Name                    | Type   | Default     |
| ----------------------- | ------ | ----------- |
| POSTGRES_USER           | String | johncena    |
| POSTGRES_PASSWORD       | String | secret1     |
| POSTGRES_DB             | String | assignments |
| POSTGRES_PORT           | uint16 | 5432        |
| POSTGRES_HOST           | String | localhost   |
| POSTGRES_MAX_CONNECTION | uint16 | 16          |

## Example

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

# Description

Please take a look inside the examples folder.All assignments are stored in PostgreSQL.
The Assignments Manager allows you to create/edit assignments.
I included some examples scripts folder in the `./examples` folder.

## How Scripts Are Tested

Each Script will run inside a docker container for max duration of 120 secs on Linux and on Windows 180 secs and the memory is limited. First the solution script will run. After that the provided script will run. At the end both stdouts and writing files are 1:1 compared

## Currently Supported Script Types

- Python3
- PowerShell\*
- Shell
- Bash
- Batch\* (windows only without docker)
- Sed
- Awk

\*Windows only because I'm using windows containers for this.

# TODO

- [x] async Tokio Command for Timeout (https://github.com/fussybeaver/bollard/pull/40)
- [x] Memory limit for script
- [x] Run each script iniside Docker (https://github.com/fussybeaver/bollard unter 4.1)
- [x] auth using some token
- [x] update testing-server status `/version`
- [x] store assignments in postgresql (tokio-postgres)
- [x] multiple rpc endpoints
- [ ] api submission better error handling on POST submission
- [ ] testing no panic if os does not support script
- [ ] testing result cache
- [ ] grpc_tester.rs see comment and fix it?!
- [ ] edit assignments remove "/r/n" on edit
- [ ] edit assignments add und edit script contains regex
- [ ] testing test script contains with some regex
- [x] assign-mngr auth
- [ ] solution script and script to test should have the same name
- [ ] files output folder shut have the same in container
