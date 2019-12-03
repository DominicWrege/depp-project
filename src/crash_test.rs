use crate::config::Error;
use crate::config::{Assignment, File, Script};
use crate::util::rm_windows_new_lines;
use failure::ResultExt;

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
//use std::time::Duration;
use log::info;
use regex::Regex;

pub fn run(assignment: &Assignment, script_path: PathBuf) -> ScriptResult {
    match inner_run(assignment, script_path.as_path()) {
        Ok(scr) => scr,
        Err(e) => {
            eprintln!("Run test Error {}", e);
            ScriptResult::InCorrect(e.to_string())
        }
    }
}

fn inner_run(assignment: &Assignment, script_path: &Path) -> Result<ScriptResult, failure::Error> {
    let script_path = add_file_extension(script_path, assignment.commandline)?;
    let output = run_script(&assignment.commandline, &script_path, &assignment.args)?;
    println!(
        "running Taskname: {} Script: {:?}",
        assignment.name, &script_path
    );
    if let Some(script_contains_with_solution) = &assignment.script_contains {
        let script_content = fs::read_to_string(&script_path).map_err(Error::IO)?;
        let script_result = if script_contains_with_solution.regex {
            file_match_line(&script_contains_with_solution.text, &script_content)?
        } else {
            contains_with_solution(&script_contains_with_solution.text, &script_content)
        };
        match script_result {
            ScriptResult::InCorrect(_) => return Ok(script_result),
            _ => (),
        }
    }
    if output.status.success() {
        if let Some(pattern) = &assignment.output {
            let stdout = String::from_utf8(output.stdout)?;
            let script_result = if pattern.regex {
                match_with_solution(&stdout, &pattern.text)?
            } else {
                contains_with_solution(&stdout, &pattern.text)
            };
            match script_result {
                ScriptResult::InCorrect(x) => return Ok(ScriptResult::InCorrect(x)),
                _ => (),
            }
        }
        match check_files(&assignment.files)? {
            ScriptResult::InCorrect(x) => return Ok(ScriptResult::InCorrect(x)),
            _ => (),
        }
    } else {
        println!(
            "Assignment: {} Script finished with exit code 1, {:?}, stderr {:?}",
            &assignment.name,
            &script_path,
            String::from_utf8(output.stderr).unwrap()
        );
        return Ok(ScriptResult::InCorrect(
            "Script finished with exit code 1".into(),
        ));
    }
    Ok(ScriptResult::Correct)
}

fn match_with_solution(stdout: &str, regex_text: &str) -> Result<ScriptResult, Error> {
    let c_regex_text = &rm_windows_new_lines(regex_text);
    let regex = Regex::new(c_regex_text).map_err(Error::Regex)?;
    let c_stdout = &rm_windows_new_lines(stdout);
    info!("Value to match: {:#?}", c_stdout);
    match regex.is_match(stdout) {
        true => Ok(ScriptResult::Correct),
        false => Ok(ScriptResult::InCorrect("Values to not match".into())),
    }
}

fn contains_with_solution(output: &str, expected_output: &str) -> ScriptResult {
    let a = rm_windows_new_lines(output.trim());
    let b = rm_windows_new_lines(expected_output.trim());
    info!("\nExpected:");
    println!("{:#?}", b);
    info!("Value:");
    println!("{:#?}", a);
    println!("--------------------------");
    //println!("compare {}", output.trim_end() == &std_solution);
    match a.contains(&b) {
        true => return ScriptResult::Correct,
        false => return ScriptResult::InCorrect("Does not Contain expected Output".into()),
    };
}

fn run_script(
    script_type: &Script,
    script_path: &Path,
    args_from_conf: &Vec<String>,
) -> Result<Output, failure::Error> {
    let (prog, mut args) = script_type.commandline();
    args.push(script_path.to_path_buf());
    let out = Command::new(prog)
        .args(args)
        .args(args_from_conf)
        .output()
        .with_context(|_| format!("Could not find script"))?;
    Ok(out)
}

fn check_files(files: &[File]) -> Result<ScriptResult, Error> {
    for x in files {
        let ret = check_file(&x.path, &x.content)?;
        match ret {
            ScriptResult::InCorrect(_) => return Ok(ret),
            _ => (),
        }
    }
    Ok(ScriptResult::Correct)
}

fn check_file(path_to_file: &PathBuf, solution: &Option<String>) -> Result<ScriptResult, Error> {
    info!("Path: {:?}", path_to_file);
    if path_to_file.exists() {
        let file_content = fs::read_to_string(path_to_file)
            .map_err(Error::IO)?
            .trim_end()
            .to_string();
        //info!("solut {:?}", solution);
        info!("file content is {:?}", &file_content);
        info!("Solution is {:?}", &solution);
        if let Some(solution) = solution {
            return Ok(contains_with_solution(&file_content, &solution));
        } else {
            return Ok(ScriptResult::Correct);
        }
    } else {
        info!("Path {:?} does not exists", path_to_file.as_os_str());
    }
    Ok(ScriptResult::InCorrect("File not found".into()))
}

fn file_match_line(regex_in: &str, script_content: &str) -> Result<ScriptResult, Error> {
    let regex = Regex::new(regex_in).map_err(Error::Regex)?;
    let c_script_content = rm_windows_new_lines(script_content);
    for line in c_script_content.lines() {
        if regex.is_match(line) {
            info!("Script contains_with_solution this pattern");
            return Ok(ScriptResult::Correct);
        }
    }
    info!("Script does not contains_with_solution this pattern");
    Ok(ScriptResult::InCorrect(
        "Script does not contains_with_solution this pattern".into(),
    ))
}

fn add_file_extension(path: &Path, script_type: Script) -> Result<PathBuf, failure::Error> {
    let ext = match script_type {
        Script::Batch => "bat",
        Script::Powershell => "ps1",
        Script::Python3 => "py",
        Script::Shell | Script::Bash => "sh",
    };
    let new_path = path.with_extension(ext);
    fs::rename(path, &new_path)?;
    Ok(new_path)
}

#[derive(Debug)]
pub enum ScriptResult {
    Correct,
    InCorrect(String),
}
impl fmt::Display for ScriptResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            ScriptResult::Correct => "Was correct".into(),
            ScriptResult::InCorrect(x) => format!("Was Incorrect because {}", x),
        };
        write!(f, " {} ", str)
    }
}
