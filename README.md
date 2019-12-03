# Description

Please take a look inside the examples folder. Inside you will find the ```exampels/assignments.yaml``` file. **Please do not move or delete this file**. 
The examples folder and the ```docs/api.openapi.yml``` will be mounted to the container. So no rebuild is necessary after change.
I also includded some example scripts you can encode to base64 for testing. Currently all ```.bat``` scripts run inside wine. This there are running slow like 4-12 seconds.

# Getting Started
## Build And Run
```
docker-compuse up
```
## Build And Run As Deamon
```
docker-compuse up -d
```
## Build Only
First build ETA is 10min.
Sometimes ```docker-compuse up``` will no rebuild some containers. So you have to do this or even with ``` --force ``` option.
```
docker-compuse build
```
## API Endpoint
[http://127.0.0.1:8080](http://127.0.0.1:8080)  
Please use the openapi 3.0 docs via swagger ui for more information how to use this api.
## Swagger UI
[http://127.0.0.1:4000](http://127.0.0.1:4000)

## Currently Supported Script Types
- Python3
- Powershell*
- Shell
- Bash
- Batch* 

*Inside Linux only core features are working.

# Assignment In More Details
Currently this a list of all assignment ```exampels/assignments.yaml``` are stored.
Here I go into more details how the assignment is structured.

```
[[assignment]]
name = "Task 9 loops in shell" 
type = "Shell"                          # see Supported Script Types 
args = ["rackrent"]                     # with this stdin I will run this script
    [assignment.output]                 # checking stdout
    regex = false                       # boolean
    text = "tnerkcar"
    [[assignment.files]]                # check if certain files/folder are created
    path = "hallo.txt"
    content = "HalloWorld"
    [[assignment.files]]
    path = "morefiles.txt"
    content = "42 answer"

```


# Discussion

- Do you want pass each request and api key?
- Do we need some auth (LTI)?

# TODO For API
- [ ]  async Tokio Command for Timeout (https://github.com/fussybeaver/bollard/pull/40) 
- [ ] Memory limit for script
- [ ] Run each script iniside Docker (https://github.com/fussybeaver/bollard unter 4.1)


