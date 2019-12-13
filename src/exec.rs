use crate::config::Script;
use std::process::{Command, Output};
use std::path::Path;
use failure::ResultExt;
pub fn run_script(
    script_type: &Script,
    script_path: &Path,
    args_from_conf: &Vec<String>,
) -> Result<Output, failure::Error>
{
    let (prog, mut args) = script_type.commandline();
    args.push(script_path.to_path_buf());
    let out = Command::new(prog)
        .args(args)
        .args(args_from_conf)
        .output()
        .with_context(|_| format!("Could not find script"))?;
    // dbg!(&out);
    Ok(out)
}


pub fn script_is_ok(out: &Output) -> bool{
    out.status.success() && out.stderr.is_empty()
}