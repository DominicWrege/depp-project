use crate::crash_test::Error;
use bollard::container::{
    CreateContainerOptions, CreateContainerResults, HostConfig, LogOutput, LogsOptions, MountPoint,
    RemoveContainerOptions, StartContainerOptions, WaitContainerOptions,
};
use futures::StreamExt;
use grpc_api::{Script, TargetOs};
use std::fmt::Write;
use std::path::Path;
use std::time::Duration;
use tokio::time::timeout;
#[derive(Debug)]
pub struct Mount<'a> {
    pub source_dir: &'a str,
    pub target_dir: &'a str,
}

enum MountPermission {
    Read,
    Write,
}

impl From<MountPermission> for Option<bool> {
    fn from(m: MountPermission) -> Option<bool> {
        let p = match m {
            MountPermission::Read => true,
            MountPermission::Write => false,
        };
        Some(p)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ImageOpt {
    pub name: &'static str,
    pub platform: &'static str,
}

pub fn docker_mount_points(script: &Script) -> (&'static str, &'static str) {
    match script.target_os() {
        TargetOs::Windows => (r"C:\testing\", r"C:\script\"),
        TargetOs::Unix => ("/testing/", "/script/"),
    }
}

#[derive(Clone, Debug)]
pub struct DockerWrap {
    docker: bollard::Docker,
    image_name: String,
}

fn create_mount_point<'a>(
    source: &'a str,
    target: &'a str,
    permission: MountPermission,
) -> MountPoint<&'a str> {
    MountPoint {
        target,
        source,
        type_: "bind",
        read_only: permission.into(),
        consistency: "default",
        ..Default::default()
    }
}

pub fn create_host_config<'a>(
    out_put_mount: &'a Mount,
    script_mount: &'a Mount,
) -> Option<HostConfig<&'a str>> {
    let output_mount_point = create_mount_point(
        out_put_mount.source_dir,
        out_put_mount.target_dir,
        MountPermission::Write,
    );
    let script_mount_point = create_mount_point(
        script_mount.source_dir,
        script_mount.target_dir,
        MountPermission::Read,
    );
    Some(HostConfig {
        mounts: Some(vec![script_mount_point, output_mount_point]),
        #[cfg(target_family = "unix")]
        memory: Some(200000000), //200MB RAM for each container
        #[cfg(target_family = "windows")]
        memory: Some(320000000), //320MB RAM for each container
        ..Default::default()
    })
}

impl From<bollard::errors::Error> for Error {
    fn from(err: bollard::errors::Error) -> Self {
        Error::Docker(err.to_string())
    }
}

// time in seconds
#[cfg(target_family = "unix")]
pub const TIMEOUT: u64 = 120;

#[cfg(target_family = "windows")]
pub const TIMEOUT: u64 = 300;

impl DockerWrap {
    pub fn new(image_name: String) -> Self {
        Self {
            docker: bollard::Docker::connect_with_local_defaults()
                .expect("Can't connect to docker api. Is the docker daemon running?"),
            image_name,
        }
    }
    pub async fn test_in_container(
        &self,
        script: &Script,
        script_path: &Path,
        out_dir: &Path,
        args_from_conf: &Vec<String>,
    ) -> Result<ScriptOutput, Error> {
        let (inner_working_dir, inner_script_dir) = docker_mount_points(script);
        let out_dir_mount = Mount {
            source_dir: out_dir.to_str().unwrap(),
            target_dir: inner_working_dir,
        };
        let script_dir_mount = Mount {
            source_dir: script_path.parent().unwrap().to_str().unwrap(),
            target_dir: inner_script_dir,
        };
        let host_config = create_host_config(&out_dir_mount, &script_dir_mount);
        let script_name = script_path.file_name().unwrap().to_str().unwrap();
        let mut cmd = script.command_line();
        let prog = format!("{}{}", inner_script_dir, script_name);
        cmd.push(prog.as_str());
        cmd.extend(args_from_conf.iter().map(|x| x.as_str()));
        let container = self
            .create_container(cmd, host_config, inner_working_dir)
            .await?;
        log::info!("Container created");
        let dur = Duration::from_secs(TIMEOUT);
        let out = timeout(dur, self.start_and_log_container(&container.id))
            .await
            .map_err(|e| {
                let err = Error::Timeout(e, dur.into());
                log::info!("{}", &err);
                err
            })?;

        self.docker
            .remove_container(
                &container.id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await?;
        log::info!("Container removed");
        dbg!(&out);
        out
    }

    pub async fn create_container(
        &self,
        cmd: Vec<&str>,
        host_config: Option<HostConfig<&str>>,
        working_dir: &str,
    ) -> Result<CreateContainerResults, bollard::errors::Error> {
        let container_config = bollard::container::Config {
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            args_escaped: None,
            image: Some(self.image_name.as_str()),
            working_dir: Some(working_dir),
            cmd: Some(cmd),
            env: None,
            stop_timeout: Some(TIMEOUT as isize),
            host_config,
            ..Default::default()
        };
        self.docker
            .create_container(None::<CreateContainerOptions<&str>>, container_config)
            .await
    }

    async fn get_output(&self, container_id: &str) -> (String, String) {
        let log_opt = Some(LogsOptions {
            stdout: true,
            stderr: true,
            ..Default::default()
        });
        let mut output_stream = self.docker.logs(container_id, log_opt);
        let mut stdout = String::new();
        let mut stderr = String::new();
        while let Some(out) = output_stream.next().await {
            match out.unwrap() {
                LogOutput::StdOut { message } => write!(stdout, "{}\n", message).unwrap(),
                LogOutput::StdErr { message } => write!(stderr, "{}\n", message).unwrap(),
                _ => (),
            }
        }
        (stdout, stderr)
    }

    pub async fn start_and_log_container(&self, container_id: &str) -> Result<ScriptOutput, Error> {
        self.docker
            .start_container(container_id, None::<StartContainerOptions<String>>)
            .await?;
        let mut wait_stream = self.docker.wait_container(
            &container_id,
            Some(WaitContainerOptions {
                condition: "not-running",
            }),
        );

        let status_code = wait_stream.next().await.unwrap().unwrap().status_code;
        let (stdout, stderr) = self.get_output(container_id).await;

        Ok(ScriptOutput {
            stdout,
            stderr,
            status_code,
        })
    }
    // TODO FIX ME error EOF
    pub async fn pull_image(&self) {
        use bollard::image::CreateImageOptions;
        let options = Some(CreateImageOptions {
            from_image: self.image_name.as_str(),
            ..Default::default()
        });
        let mut stream = self.docker.create_image(options, None);
        log::info!("pulling {}", self.image_name);
        while let Some(s) = stream.next().await {
            if let Err(err) = s {
                log::error!(
                    "Could pull image: {}. Maybe because it was not found.",
                    &self.image_name
                );
                panic!(
                    "Could not pull docker image: {}, err: {}",
                    self.image_name, err
                );
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScriptOutput {
    pub stdout: String,
    pub stderr: String,
    pub status_code: u64,
}
