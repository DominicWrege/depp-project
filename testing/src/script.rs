use crate::crash_test::Error;
use grpc_api::Script;
use std::convert::TryFrom;
use std::path::Path;
use std::process::Output;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

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

pub async fn run(
    script: &Script,
    script_path: &Path,
    dir: &Path,
    args_from_conf: &Vec<String>,
) -> Result<ScriptOutput, Error> {
    let (prog, mut args) = script.command_line();
    let dur = Duration::from_secs(30);

    #[cfg(target_family = "windows")]
    {
        args.push(fix_windows_path(&script, &script_path));
    }

    #[cfg(target_family = "unix")]
    {
        args.push(script_path.to_path_buf());
    }
    dbg!(&args);
    let out = timeout(
        dur,
        Command::new(prog)
            .current_dir(dir)
            .args(args)
            .args(args_from_conf)
            .output(),
    )
    .await
    .map_err(|e| Error::Timeout(e, dur.into()))?;
    dbg!(&out);
    let out = match out {
        Err(_) => panic!("Command {} not found!", prog),
        Ok(out) => out,
    };
    exited_fine(&out)?;
    ScriptOutput::try_from(out)
}

impl TryFrom<Output> for ScriptOutput {
    type Error = Error;

    fn try_from(o: Output) -> Result<Self, Error> {
        Ok(ScriptOutput {
            stdout: String::from_utf8(o.stdout.clone())?,
            output: o,
        })
    }
}
pub struct ScriptOutput {
    pub stdout: String,
    pub output: Output,
}

fn exited_fine(out: &Output) -> Result<(), Error> {
    if out.status.success() && out.stderr.is_empty() {
        Ok(())
    } else {
        Err(Error::ExitCode(
            String::from_utf8(out.stderr.clone()).unwrap_or_default(),
        ))
    }
}
