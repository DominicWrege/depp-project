use std::path::Path;
use std::process::Output;
use std::time::Duration;

use crate::config::Script;
use crate::crash_test::Error;
use tokio::process::Command;
use tokio::time::timeout;

// TODO print exec command with all args
//     let a = args
//     .iter()
//     .filter_map(|path| path.to_str())
//     .collect::<Vec<_>>()
//     .join("\n");
// println!(
//     "Executing: {} {} {}",
//     &prog,
//     a.trim(),
//     &args_from_conf.join(" ").trim()

// );

pub async fn run_script(
    script_type: &Script,
    script_path: &Path,
    args_from_conf: &Vec<String>,
) -> Result<Output, Error> {
    let (prog, mut args) = script_type.commandline();
    let dur = Duration::from_secs(30);
    args.push(script_path.to_path_buf());
    let out = timeout(
        dur,
        Command::new(prog)
            .current_dir("/tmp")
            .args(args)
            .args(args_from_conf)
            .output(),
    )
    .await
    .map_err(|e| Error::Timeout(e, dur.into()))?;
    match out {
        Err(_) => panic!("Command {} not found!", prog),
        Ok(out) => Ok(out),
    }
}

/*
pub trait FutureExt: std::future::Future + Sized  {
    fn timeout(self, timeout: std::time::Duration) -> tokio::time::Timeout<Self> {
        tokio::time::timeout(timeout, self)
    }
}*/

pub fn script_exit_for_out(out: &Output) -> Result<(), Error> {
    if out.status.success() && out.stderr.is_empty() {
        Ok(())
    } else {
        Err(Error::ExitCode(
            String::from_utf8(out.stderr.clone()).unwrap_or_default(),
        ))
    }
}
