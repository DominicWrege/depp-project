# Description

Please take a look inside the examples folder. Inside you will find the `exampels/assignments.yaml` file. Please do  **not move or delete this file**.
The examples folder and the `docs/api.openapi.yml` will be mounted to the container. **Note**: The server reads the ```assignments.yaml``` file after start **once**. So a restart is necessary after change to take effect. I also includded some example scripts you can encode to base64 for testing.

### Batch files On Linux

Currently all Batch files are running inside wine on Linux. This means each script takes around 4-15 seconds.

# Getting Started

## Build And Run

```
docker-compose up
```

## Build And Run As Deamon

```
docker-compose up -d
```

## Build Only

First build ETA is 10min.
Sometimes `docker-compose up` will no rebuild some containers. So you have to do this or even with `--force` option.

```
docker-compose build
```

## API Endpoint

[http://127.0.0.1:8080](http://127.0.0.1:8080)  
Please use the openapi 3.0 docs via swagger ui for more information how to use this api.

## Swagger UI

[http://127.0.0.1:4000](http://127.0.0.1:4000)

## Currently Supported Script Types

-   Python3
-   PowerShell\*
-   Shell
-   Bash
-   Batch\*

\*Inside Linux only core features are working.

# Assignment In More Details

Currently this a list of all assignment `exampels/assignments.yaml` are stored.
Here I go into more details how the assignment is structured.

```
[[assignment]]
name = "Task 9 loops in shell"              # required
type = "Shell"                              # required, set the script type
args = ["rackrent"]                         # optional, a list arguments passed to the script
    [assignment.script-contains]            # optinal, locking for pattern inside the script with an regex or contains
    regex = true                            # optinal, default is false
    text: "^echo [0-9][0-9]$"               # can be an regex if regex is set to true
    [assignment.output]                     # optional, checking stdout
    regex = false                           # optinal, default is false
    text = "tnerkcar"                       # can be an regex if regex is set to true
    [[assignment.files]]                    # optional, if certain files/folders are created
    path = "hallo.txt"
    content = "HalloWorld"
    [[assignment.files]]
    path = "morefiles.txt"
    content = "42 answer"

```

# Discussion

-   Do you want pass each request an API key?
-   Do we need some auth (LTI)?

# TODO For API

-   [ ] async Tokio Command for Timeout (https://github.com/fussybeaver/bollard/pull/40)
-   [ ] Memory limit for script
-   [ ] Run each script iniside Docker (https://github.com/fussybeaver/bollard unter 4.1)
-   [ ] remove pause keyword from bat scripts
