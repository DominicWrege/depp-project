use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::time;

use crate::base64::Base64;
use crate::config::{Assignment, File};
use crate::crash_test::Error::RequiredFileNotFound;
use crate::fs_util::{cp_include_into, new_tmp_script_file};
use log::info;
use walkdir::WalkDir;

pub trait Tester {
    fn test(&self) -> Result<(), Error>;
}
pub struct Stdout {
    expected: String,
    std_out: String,
}
pub struct Files {
    files: Vec<File>,
    //files: Vec<PathBuf>,
}
//struct Contains;

impl Tester for Stdout {
    fn test(&self) -> Result<(), Error> {
        fz_compare_with_solution(&self.std_out, &self.expected)
    }
}

impl Tester for Files {
    fn test(&self) -> Result<(), Error> {
        for file in &self.files {
            check_file(&file.path, "Not ready jet".into())?;
        }
        Ok(())
    }
}

impl Stdout {
    fn boxed(expected: String, std_out: String) -> Box<dyn Tester> {
        Box::new(Stdout { expected, std_out })
    }
}

impl Files {
    fn boxed(files: Vec<File>) -> Box<dyn Tester> {
        //        let files = WalkDir::new(dir)
        //            .into_iter()
        //            .map(|e| e.unwrap().into_path())
        //            .collect::<Vec<_>>();
        Box::new(Files { files })
    }
}

pub async fn run(assignment: &Assignment, code: &Base64) -> Result<(), Error> {
    let dir_to_test = tempfile::tempdir()?;
    let dir_solution = tempfile::tempdir()?;

    let script_test_path = new_tmp_script_file(assignment.script_type, code)
        .map_err(Error::CantCreatTempFile)?
        .into_temp_path();
    // TODO fix unwrap
    let script_solution_path = dir_solution
        .path()
        .join(&assignment.solution_path.as_path().file_name().unwrap());
    fs::copy(&assignment.solution_path, &script_solution_path)?;
    cp_include_into(&assignment.include_files, &dir_solution, &dir_to_test)?;

    let test_output = assignment
        .script_type
        .run(&script_test_path, &dir_to_test.path(), &assignment.args)
        .await?;
    info!("running task: {}", &assignment.name);
    let solution_output = assignment
        .script_type
        .run(
            &script_solution_path,
            &dir_solution.path(),
            &assignment.args,
        )
        .await?;
    let mut tests: Vec<Box<dyn Tester>> = Vec::new();
    tests.push(Stdout::boxed(solution_output.stdout, test_output.stdout));
    tests.push(Files::boxed(assignment.files.clone()));
    tests
        .iter()
        .map(|item| item.test())
        .collect::<Result<_, _>>()
}

fn fz_compare_with_solution(stdout: &str, expected_output: &str) -> Result<(), Error> {
    let stdout = trim_new_lines(stdout);
    let expected_output = trim_new_lines(expected_output);
    log::info!("\nexpected: {:#?}\nvalue: {:#?}", expected_output, stdout);
    if expected_output.contains(&stdout) {
        Ok(())
    } else {
        Err(Error::WrongOutput(format!(
            "expected: ({:#?}) value: ({:#?})",
            expected_output, stdout
        )))
    }
}

fn check_file(path_to_file: &PathBuf, solution: &str) -> Result<(), Error> {
    info!("path: {:?}", path_to_file);
    if path_to_file.exists() {
        let file_content = fs::read_to_string(path_to_file)
            .map_err(|e| Error::ReadFile(e, path_to_file.into()))?
            .trim_end()
            .to_string();
        info!("file content is {:?}", &file_content);
        info!("solution is {:?}", &solution);
        fz_compare_with_solution(&file_content, &solution)
    } else {
        Err(RequiredFileNotFound(path_to_file.to_path_buf()))
    }
}

#[derive(Debug, err_derive::Error, derive_more::From)]
pub enum Error {
    #[error(display = "IO error while reading the file: {:#?}, {:#?}", _1, _0)]
    ReadFile(std::io::Error, PathBuf),
    #[error(display = "Time out reached! Script took more than {}.", _1)]
    Timeout(tokio::time::Elapsed, DurationDisplay),
    #[from]
    #[error(display = "Script produced invalid UFT8.")]
    NoUTF8(std::string::FromUtf8Error),
    #[error(display = "Does not contains expected output. {}", _0)]
    WrongOutput(String),
    #[error(display = "Required file not found. Path {:#?} does not exists", _0)]
    RequiredFileNotFound(PathBuf),
    #[error(display = "Script finished with exit code 1 stderr: {}", _0)]
    ExitCode(String),
    #[from]
    #[error(display = "Can't create temp file. {}", _0)]
    CantCreatTempFile(std::io::Error),
    #[from]
    #[error(display = "Could not copy included files for testing {}", _0)]
    Copy(fs_extra::error::Error),
}
#[derive(Debug, derive_more::From)]
pub struct DurationDisplay(time::Duration);

impl fmt::Display for DurationDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} seconds", self.0.as_secs())
    }
}

pub fn trim_new_lines(s: &str) -> String {
    s.chars()
        .filter(|&c| c != '\r')
        .collect::<String>()
        .lines()
        .map(|line| {
            let mut n_line = line.trim_end().to_string();
            n_line.push('\n');
            n_line
        })
        .collect::<String>()
}

// TODO redo check output with regex or contains
