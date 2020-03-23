
# Usage

## Basic Auth 

The REST API uses basic auth passed via environment variable. Docker-compose requires a ```./example.env``` file that contains all needed credentials.  
#TODO update

Defualt config ```./example.env```:

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
Default: API_USERNAME=user, API_PASSWORD=wasd4221

#TODO update

## Run

The testing and api server are booth prebuilt available on dockerhub.
So now it should take a couple seconds.

```
$ docker-compose up
```

## API Endpoint
Local
[http://localhost:6000](http://localhost:6000) (See Swagger UI for more information.)  
Remote
[https://auth.inf.fh-dortmund.de:2443/api](https://auth.inf.fh-dortmund.de:2443/api)


## Assignments Manager
Local
[http://localhost:5000](http://localhost:4000)  
Remote
[https://auth.inf.fh-dortmund.de:2443/manage](https://auth.inf.fh-dortmund.de:2443/manage)

## Swagger UI
Local
[http://localhost:4000](http://localhost:4000)

# Description

Please take a look inside the examples folder. Inside you will find the `exampels/assignments.yaml` file. Please do **not move or delete this file**.
The examples folder and the `docs/api.openapi.yml` will be mounted to the container. **Note**: The server reads the `assignments.yaml` file after start **once**. So a restart is necessary after change to take effect. I also included some example scripts you can encode to base64 for testing.

## How Scripts Are Tested
Each Script will run inside a docker container for max duration of 120 secs and 
the memory is limited to 256MB. First the solution script will run. After that the provided script will run. At the end both stdouts and writing files are 1:1 compared

## Currently Supported Script Types

- Python3
- PowerShell\*
- Shell
- Bash
- Batch\* (windows only without docker)
- Sed
- Awk

\*Windows only because I'm using windows containers for this.

# Assignment In More Details

Currently this a list of all assignment `exampels/assignments.yaml` are stored.
Here I go into more details how the assignment is structured. use db-edit crate. TODO


# TODO

- [x] async Tokio Command for Timeout (https://github.com/fussybeaver/bollard/pull/40)
- [x] Memory limit for script
- [x] Run each script iniside Docker (https://github.com/fussybeaver/bollard unter 4.1)
- [x] auth using some token
- [x] update testing-server status `/version`
- [x] store assignments in postgresql (tokio-postgres)
- [x] multiple rpc endpoints
- [ ] TLS for grpc
- [ ] test ruslt cache
- [ ] edit assignments
- [ ] db-edit auth