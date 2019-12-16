use crate::base64::Base64;
use crate::config::{Assignment, File};
use crate::crash_test::Error::RequiredFileNotFound;
use crate::exec::{run_script, script_exit_fine};
use crate::fs_util::new_tmp_script_file;
use crate::util::rm_windows_new_lines;
use log::info;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time;

pub struct Out {
    std_out: String,
    //script_path: Option<PathBuf>
}

pub trait Tester {
    fn test(&self, p: &Out) -> Result<(), Error>;
}
pub struct Stdout {
    expected: String,
}
pub struct Files {
    files: Vec<File>,
}
//struct Contains;

impl Tester for Stdout {
    fn test(&self, p: &Out) -> Result<(), Error> {
        contains_with_solution(&p.std_out, &self.expected)
    }
}

impl Tester for Files {
    fn test(&self, _p: &Out) -> Result<(), Error> {
        for file in &self.files {
            check_file(&file.path, &file.content)?;
        }
        Ok(())
    }
}

impl Stdout {
    fn boxed(expected: String) -> Box<dyn Tester> {
        Box::new(Stdout { expected })
    }
}

impl Files {
    fn boxed(files: Vec<File>) -> Box<dyn Tester> {
        Box::new(Files { files })
    }
}

pub async fn run(assignment: &Assignment, code: &Base64) -> Result<(), Error> {
    let script_path = new_tmp_script_file(assignment.commandline, code)
        .map_err(Error::CantCreatTempFile)?
        .into_temp_path();
    let output = run_script(&assignment.commandline, &script_path, &assignment.args).await?;
    log_running_task(&assignment.name, &script_path);
    let solution_output = run_script(
        &assignment.commandline,
        &assignment.solution_path,
        &assignment.args,
    )
    .await?;
    log_running_task(&assignment.name, &assignment.solution_path);
    if script_exit_fine(&output) {
        let mut tests: Vec<Box<dyn Tester>> = Vec::new();

        let payload = Out {
            std_out: String::from_utf8(output.stdout)?,
            /*          script_path: None,*/
        };
        tests.push(Stdout::boxed(
            String::from_utf8(solution_output.stdout).unwrap(),
        ));
        tests.push(Files::boxed(assignment.files.clone()));

        tests
            .iter()
            .map(|item| item.test(&payload))
            .collect::<Result<_, _>>()
    } else {
        Err(Error::ExitCode(
            String::from_utf8(output.stderr).unwrap_or_default(),
        ))
    }
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
}

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

fn contains_with_solution(output: &str, expected_output: &str) -> Result<(), Error> {
    let std_out = rm_windows_new_lines(output.trim());
    let expected_output = rm_windows_new_lines(expected_output.trim());
    log::info!("\nExpected: {:#?} Value: {:#?}", expected_output, std_out);
    if std_out.contains(&expected_output) {
        Ok(())
    } else {
        Err(Error::WrongOutput)
    }
}

fn check_file(path_to_file: &PathBuf, solution: &str) -> Result<(), Error> {
    info!("Path: {:?}", path_to_file);
    if path_to_file.exists() {
        let file_content = fs::read_to_string(path_to_file)
            .map_err(|e| Error::ReadFile(e, path_to_file.into()))?
            .trim_end()
            .to_string();
        println!("file content is {:?}", &file_content);
        println!("Solution is {:?}", &solution);
        contains_with_solution(&file_content, &solution)
    } else {
        Err(RequiredFileNotFound(path_to_file.to_path_buf()))
    }
}

/*fn file_match_line(regex_in: &str, script_content: &str) -> Result<ScriptResult, Error> {
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
}*/

/*fn check_script_content(script_path: &Path, pattern: &Pattern) -> Result<ScriptResult, Error> {
    let script_content = fs::read_to_string(&script_path)
        .map_err(|e| Error::ReadFile(e, script_path.to_path_buf()))?;
    let script_result = if pattern.regex {
        file_match_line(&pattern.text, &script_content)?
    } else {
        contains_with_solution(&pattern.text, &script_content)
    };
    Ok(script_result)
}*/

#[derive(Debug, err_derive::Error, derive_more::From)]
pub enum Error {
    #[error(display = "IO error while reading the file: {:#?}, {:#?}", _1, _0)]
    ReadFile(std::io::Error, PathBuf),
    #[error(display = "Time out reached! Script took more than {}.", _1)]
    Timeout(tokio::time::Elapsed, DurationDisplay),
    #[from]
    #[error(display = "Script produced invalid UFT8.")]
    NoUTF8(std::string::FromUtf8Error),
    #[error(display = "Does not contains expected output")]
    WrongOutput,
    #[error(display = "Required file not found. Path {:#?} does not exists", _0)]
    RequiredFileNotFound(PathBuf),
    #[error(display = "Script finished with exit code 1 stderr: {}", _0)]
    ExitCode(String),
    #[error(display = "Can't create temp file. {}", _0)]
    CantCreatTempFile(std::io::Error),
}
#[derive(Debug, derive_more::From)]
pub struct DurationDisplay(time::Duration);

impl fmt::Display for DurationDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} seconds", self.0.as_secs())
    }
}

fn log_running_task(name: &str, path: &Path) {
    println!("running Taskname: {} Script: {:?}", name, path);
}
