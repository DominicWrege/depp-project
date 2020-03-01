use crate::crash_test::Error;
use crate::docker_api::{
    create_container, create_host_config, docker_image, docker_mount_points,
    start_and_log_container, Mount,
};
use bollard::container::RemoveContainerOptions;
use grpc_api::Script;
use std::path::Path;
use std::time::Duration;
use tokio::time::timeout;

pub const TIMEOUT: u64 = 120;

pub async fn run_in_container(
    docker: &bollard::Docker,
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
    let container = create_container(
        cmd,
        docker_image(&script).name,
        host_config,
        inner_working_dir,
        &docker,
    )
    .await?;
    log::info!("Container created");
    let dur = Duration::from_secs(TIMEOUT);
    let out = timeout(dur, start_and_log_container(&container.id, &docker))
        .await
        .map_err(|e| {
            let err = Error::Timeout(e, dur.into());
            log::info!("{}", &err);
            err
        })?;

    docker
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

#[derive(Debug, Clone)]
pub struct ScriptOutput {
    pub stdout: String,
    pub stderr: String,
    pub status_code: u64,
}

/*impl TryFrom<Output> for ScriptOutput {
    type Error = Error;

    fn try_from(o: Output) -> Result<Self, Error> {
/*        Ok(ScriptOutput {
            stdout: String::from_utf8(o.stdout.clone())?,
            o*/
        })
    }
}*/
/*fn exited_fine(out: &Output) -> Result<(), Error> {
    if out.status.success() && out.stderr.is_empty() {
        Ok(())
    } else {
        Err(Error::ExitCode(
            String::from_utf8(out.stderr.clone()).unwrap_or_default(),
        ))
    }
}*/

#[cfg(target_family = "windows")]
fn fix_windows_path(script: &Script, script_path: &Path) -> std::ffi::OsString {
    use path_slash::PathExt;
    use regex::{Captures, Regex};
    if script == &Script::Bash || script == &Script::Shell {
        let str = script_path.to_slash_lossy().replace("\\\\?\\", "");
        let re = Regex::new(r"^([A-Z])://").unwrap();
        re.replace(&str, |caps: &Captures| {
            format!("/mnt/{}/", caps[1].to_ascii_lowercase())
        })
        .to_string()
        .into()
    } else {
        script_path.into()
    }
}
