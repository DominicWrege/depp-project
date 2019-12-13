use std::path::Path;
use std::process::Output;
use std::time::Duration;

use tokio::time::timeout;
use failure::ResultExt;
use tokio::process::Command;
use crate::config::Script;

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
) -> Result<Output, failure::Error> {
    let (prog, mut args) = script_type.commandline();
    args.push(script_path.to_path_buf());
    let out = timeout(Duration::from_secs(30), Command::new(prog)
        .args(args)
        .args(args_from_conf)
        .output())
        .await
        // FIX me
        .with_context(|_| format!("30 seconds time out for script reached"))?
        .with_context(|_| format!("Could not find script"))?;
    Ok(out)
}

/*

    // dbg!(&out);

pub trait FutureExt: std::future::Future + Sized  {
    fn timeout(self, timeout: std::time::Duration) -> tokio::time::Timeout<Self> {
        tokio::time::timeout(timeout, self)
    }
}*/

pub fn script_is_ok(out: &Output) -> bool {
    out.status.success() && out.stderr.is_empty()
}
