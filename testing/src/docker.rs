use crate::script::ScriptOutput;
use bollard::container::{
    CreateContainerOptions, CreateContainerResults, HostConfig, LogOutput, LogsOptions, MountPoint,
    StartContainerOptions,
};
use futures::StreamExt;
use std::fmt::Write;

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
    /*    dbg!(&script_mount_point);
    dbg!(&output_mount_point);*/
    Some(HostConfig {
        mounts: Some(vec![script_mount_point, output_mount_point]),
        ..Default::default()
    })
}

pub async fn create_container(
    working_dir: &str,
    script_name: &str,
    host_config: Option<HostConfig<&str>>,
    docker: &bollard::Docker,
    script: &grpc_api::Script,
    args_from_conf: &Vec<String>,
) -> Result<CreateContainerResults, bollard::errors::Error> {
    let (prog, _args) = script.command_line(); //TODO rm _args
    let arg1 = ["/script_dir/".as_ref(), script_name].join("");
    let mut cmd = vec![prog, arg1.as_ref()];
    let mut args2: Vec<&str> = args_from_conf.iter().map(AsRef::as_ref).collect();
    cmd.append(args2.as_mut());
    //dbg!(&cmd);

    let image = if script == &grpc_api::Script::Python3 {
        Some("my_python3")
    } else {
        Some("ubuntu:latest")
    };

    let container_config = bollard::container::Config {
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        args_escaped: None,
        image: image,
        working_dir: Some(working_dir),
        cmd: Some(cmd),
        env: None,
        host_config,
        ..Default::default()
    };
    docker
        .create_container(None::<CreateContainerOptions<&str>>, container_config)
        .await
}

pub async fn start_container(container_id: &str, docker: &bollard::Docker) -> ScriptOutput {
    docker
        .start_container(container_id, None::<StartContainerOptions<String>>)
        .await
        .expect("error start container");
    get_out_put(container_id, &docker).await
}

async fn get_out_put(container_id: &str, docker: &bollard::Docker) -> ScriptOutput {
    let log_opt = Some(LogsOptions {
        stdout: true,
        stderr: true,
        ..Default::default()
    });
    let mut output_stream = docker.logs(container_id, log_opt);
    let mut stdout = String::new();
    let mut stderr = String::new();
    while let Some(out) = output_stream.next().await {
        match out.unwrap() {
            LogOutput::StdOut { message } => write!(stdout, "{}\n", message).unwrap(),
            LogOutput::StdErr { message } => write!(stderr, "{}\n", message).unwrap(),
            _ => (),
        }
    }
    ScriptOutput { stdout, stderr }
}
