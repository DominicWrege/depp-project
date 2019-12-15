use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time;

use log::info;
use regex::Regex;

use crate::config::{Assignment, File, Pattern};
use crate::exec::{run_script, script_exit_fine};
use crate::util::rm_windows_new_lines;
use std::process::Output;

pub async fn run(assignment: &Assignment, script_path: PathBuf) -> ScriptResult {
    match inner_run(assignment, script_path.as_path()).await {
        Ok(scr) => scr,
        Err(e) => {
            eprintln!("Run test Error {}", e);
            ScriptResult::InCorrect(e.to_string())
        }
    }
}

async fn inner_run(
    assignment: &Assignment,
    script_path: &Path,
) -> Result<ScriptResult, failure::Error> {
    let output = run_script(&assignment.commandline, &script_path, &assignment.args).await?;
    log_runing_task(&assignment.name, &script_path);
    let solution_output = run_script(
        &assignment.commandline,
        &assignment.solution_path,
        &assignment.args,
    )
    .await?;
    log_runing_task(&assignment.name, &assignment.solution_path);
    if script_exit_fine(&output) {
        // TODO FIX ME
        if !solution_output.stdout.is_empty() {
            let _a = check_stdout_with_solution(&output, &solution_output)?;
        }
        // TODO FIX ME
        let _tmp1_script_result = if let Some(pattern) = &assignment.script_contains {
            check_script_content(&script_path, &pattern)?
        } else {
            ScriptResult::Correct
        };
        /*
         // TODO redo check output with regex or contains
        let tmp2_script_result = if pattern.regex {
              match_with_solution(&stdout, &pattern.text)?
          } else {
              contains_with_solution(&stdout, &pattern.text)
          };
          match tmp2_script_result {
              ScriptResult::InCorrect(x) => return Ok(ScriptResult::InCorrect(x)),
              _ => (),
          }*/
        if let ScriptResult::InCorrect(x) = check_files(&assignment.files)? {
            return Ok(ScriptResult::InCorrect(x));
        }
    } else {
        let err_msg = format!(
            "Script finished with exit code 1, {}, stderr {}",
            &assignment.name,
            String::from_utf8(output.stderr).unwrap_or_default()
        );
        println!("{:?}: {}", &script_path, err_msg);
        return Ok(ScriptResult::InCorrect(err_msg));
    }
    Ok(ScriptResult::Correct)
}

// TODO Maybe use later
/*fn match_with_solution(stdout: &str, regex_text: &str) -> Result<ScriptResult, Error> {
    let c_regex_text = &rm_windows_new_lines(regex_text);
    let regex = Regex::new(c_regex_text)?;
    let c_stdout = &rm_windows_new_lines(stdout);
    info!("Value to match: {:#?}", c_stdout);
    match regex.is_match(stdout) {
        true => Ok(ScriptResult::Correct),
        false => Ok(ScriptResult::InCorrect("Values to not match".into())),
    }
}*/

// better error output maybe in master
fn contains_with_solution(output: &str, expected_output: &str) -> ScriptResult {
    let a = rm_windows_new_lines(output.trim());
    let b = rm_windows_new_lines(expected_output.trim());
    println!("\nExpected:");
    println!("{:#?}", b);
    println!("Value:");
    println!("{:#?}", a);
    println!("--------------------------");
    //println!("compare {}", output.trim_end() == &std_solution);
    match a.contains(&b) {
        true => return ScriptResult::Correct,
        false => return ScriptResult::InCorrect("Does not Contain expected Output".into()),
    };
}

fn check_stdout_with_solution(
    output: &Output,
    solution_output: &Output,
) -> Result<ScriptResult, Error> {
    println!("------------------------");
    // TODO why output.stdout.clone() and solution_output.stdout.clone() why clone  ??
    let stdout = String::from_utf8(output.stdout.clone())?;
    dbg!(&stdout);
    println!("######################");
    println!("######################");
    let stdout_solution = String::from_utf8(solution_output.stdout.clone())?;
    dbg!(&stdout_solution);
    println!("------------------------");
    let res = contains_with_solution(&stdout, &stdout_solution);
    Ok(res)
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
            .map_err(|e| Error::ReadFile(e, path_to_file.into()))?
            .trim_end()
            .to_string();
        //info!("solut {:?}", solution);
        println!("file content is {:?}", &file_content);
        println!("Solution is {:?}", &solution);
        if let Some(solution) = solution {
            return Ok(contains_with_solution(&file_content, &solution));
        } else {
            return Ok(ScriptResult::Correct);
        }
    } else {
        println!("Path {:?} does not exists", path_to_file.as_os_str());
    }
    Ok(ScriptResult::InCorrect("File not found".into()))
}

fn file_match_line(regex_in: &str, script_content: &str) -> Result<ScriptResult, Error> {
    let regex = Regex::new(regex_in)?;
    let c_script_content = rm_windows_new_lines(script_content);
    for line in c_script_content.lines() {
        if regex.is_match(line) {
            info!("Script contains_with_solution this pattern");
            return Ok(ScriptResult::Correct);
        }
    }
    println!("Script does not contains_with_solution this pattern");
    Ok(ScriptResult::InCorrect(
        "Script does not contains_with_solution this pattern".into(),
    ))
}

fn check_script_content(script_path: &Path, pattern: &Pattern) -> Result<ScriptResult, Error> {
    let script_content = fs::read_to_string(&script_path)
        .map_err(|e| Error::ReadFile(e, script_path.to_path_buf()))?;
    let script_result = if pattern.regex {
        file_match_line(&pattern.text, &script_content)?
    } else {
        contains_with_solution(&pattern.text, &script_content)
    };
    Ok(script_result)
}

#[derive(Debug, err_derive::Error, derive_more::From)]
pub enum Error {
    #[error(display = "IO error while reading the file: {:#?}, {:#?}", _1, _0)]
    ReadFile(std::io::Error, PathBuf),
    #[error(display = "Command {} not found", _0)]
    CommandNotFound(String),
    #[from]
    #[error(display = "Wrong regex was, {}", _0)]
    Regex(regex::Error),
    #[error(display = "Time out reached! Script took more than {}.", _1)]
    Timeout(tokio::time::Elapsed, DurationDisplay),
    #[from]
    #[error(display = "No valid UFT8.")]
    NoUTF8(std::string::FromUtf8Error),
}
#[derive(Debug, derive_more::From)]
pub struct DurationDisplay(time::Duration);

impl fmt::Display for DurationDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} seconds", self.0.as_secs())
    }
}

fn log_runing_task(name: &str, path: &Path) {
    println!("running Taskname: {} Script: {:?}", name, path);
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
