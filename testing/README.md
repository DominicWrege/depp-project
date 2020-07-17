## Description

A RPC server waiting to test scripts inside a Docker container.
Resources like time and memory are limited. Each container gets a specific folder mounted where the fs output
is been checked alongside with the console output as well. You need to run two instances (Windows and Linux) in order to test all scripts.

## Build and Run In Debug Mode

```
cargo run
```

## Config

| Name                   | Type                | Default                                                                                          |
| ---------------------- | ------------------- | ------------------------------------------------------------------------------------------------ |
| DEPP_TEST_PORT         | uint16              | 50051                                                                                            |
| DEPP_TEST_MAX_CURR     | uint8               | Linux: 10, Windows: 5                                                                            |
| DEPP_TEST_DOCKER_IMAGE | String              | Linux: `dominicwrege/depp-project-ubuntu:latest`, Windows: `mcr.microsoft.com/powershell:latest` |
| DEPP_TEST_TIMEOUT      | uint64 format: secs | Linux: 120, Windows: 180                                                                         |

## Deploy

### Linux

Run the testing server [dominicwrege/depp-testing](https://hub.docker.com/r/dominicwrege/depp-testing) using Docker and by mounting the
daemon: `/var/run/docker.sock:/var/run/docker.sock` as an volume.

### Windows

On Windows to just compile and run it directly on the host machine.
For setting the environment variables [git Windows Bash](https://git-scm.com/download/win) is also recommended.

##### Choose the docker testing image:

```
# Windows 10 Pro/Enterprise
export DEPP_TEST_DOCKER_IMAGE=mcr.microsoft.com/powershell:latest
# or
# Windows Server 2016
export DEPP_TEST_DOCKER_IMAGE=mcr.microsoft.com/windows/servercore:ltsc2016
# or
# Windows Server 2019
export DEPP_TEST_DOCKER_IMAGE=mcr.microsoft.com/windows/servercore:ltsc2019
```

##### Compile and run:

```
cargo run --release
```

## Docker Images Used For Testing The Scripts

- Python3
- Shell
- Bash
- Sed
- Awk

These scripts are tested using this Docker image [dominicwrege/depp-project-ubuntu](https://hub.docker.com/r/dominicwrege/depp-project-ubuntu).

- PowerShell
- Batch

These scripts are tested using this Docker image [mcr.microsoft.com/powershell](https://hub.docker.com/_/microsoft-powershell).  
**Note**: Docker images on Windows Server require to have matching kernel version with the host machine.
For example can't run [mcr.microsoft.com/windows/servercore:ltsc2019 (Windows Server 2019)](https://hub.docker.com/_/microsoft-windows-servercore) image on a
Windows Server 2016 host.

### Recommended Windows Docker Images

- Windows 10 Pro/Enterprise: `mcr.microsoft.com/powershell`
- Windows Server 2016: `mcr.microsoft.com/windows/servercore:ltsc2016`
- Windows Server 2019: `mcr.microsoft.com/windows/servercore:ltsc2019`

For a full list please checkout [Windows container version compatibility](https://docs.microsoft.com/en-us/virtualization/windowscontainers/deploy-containers/version-compatibility?tabs=windows-server-2004%2Cwindows-10-2004).

## Build And Publish The Docker Image

```
sudo docker build -t testing -f ../Docker-Files/Dockerfile-Testing ..
sudo docker tag testing dominicwrege/depp-project-testing:latest
sudo docker push dominicwrege/depp-project-testing
```
