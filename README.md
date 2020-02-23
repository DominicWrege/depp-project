
# Usage

## Basic Auth 

The REST API uses basic auth passed via environment variable. Docker-compose requires a ```.env``` file that contains all needed credentials.
username: ```API_USERNAME``` password: ```API_PASSWORD```
```
$ echo "API_USERNAME=tester API_PASSWORD=password42 > .env"
```
## Run

The testing and api server are booth prebuilt available on dockerhub.
So now it should take a couple seconds.

```
$ docker-compose up
```

## API Endpoint

[http://127.0.0.1:8080](http://127.0.0.1:8080)  
Please use the openapi 3.0 docs via swagger ui for more information how to use this api.

## Swagger UI

[http://127.0.0.1:4000](http://127.0.0.1:4000)

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
Here I go into more details how the assignment is structured.

```
[[assignment]]
name = "Task 9 loops in shell"              # required
type = "Shell"                              # required, set the script type
args = ["rackrent"]                         # optional, a list arguments passed to the
include-files = ["examples/akademisches_jahrbuch.txt"] # optional inlcude needed files
solution = '''#!/bin/bash echo HelloWorld''' # required sample solution
```

# TODO

- [x] async Tokio Command for Timeout (https://github.com/fussybeaver/bollard/pull/40)
- [x] Memory limit for script
- [x] Run each script iniside Docker (https://github.com/fussybeaver/bollard unter 4.1)
- [ ] remove pause keyword from bat scripts?!
- [x] auth using some token
- [x] update testing-server status `/version`
- [ ] store assignments in postgresql (tokio-postgres)
- [ ] multiple rpc endpoints