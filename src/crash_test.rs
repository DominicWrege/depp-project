use std::path::{Path, PathBuf};
use std::time;
use std::{fmt, fs};

use crate::base64::Base64;
use crate::config::Assignment;
use crate::fs_util::{cp_include_into, new_tmp_script_file};
use log::info;
use walkdir::WalkDir;
//use walkdir::WalkDir;

pub trait Tester {
    fn test(&self) -> Result<(), Error>;
}
pub struct Stdout {
    expected: String,
    std_out: String,
}
pub struct Files {
    expected_dir: PathBuf,
    given_dir: PathBuf,
}
//struct Contains;

impl Tester for Stdout {
    fn test(&self) -> Result<(), Error> {
        let stdout = trim_new_lines(&self.std_out);
        let expected_output = trim_new_lines(&self.expected);
        log::info!("expected: {:#?}", expected_output);
        log::info!("result: {:#?}", stdout);
        if expected_output.contains(&stdout) {
            Ok(())
        } else {
            Err(Error::WrongOutput(format!(
                "expected: ({:#?}) result: ({:#?})",
                expected_output, stdout
            )))
        }
    }
}

impl Stdout {
    fn boxed(expected: String, std_out: String) -> Box<dyn Tester> {
        Box::new(Stdout { expected, std_out })
    }
}

// Booth dirs have to have exact the same content
// Maybe use your own impl
impl Tester for Files {
    fn test(&self) -> Result<(), Error> {
        print_dir_content("expected dir:", &self.expected_dir);
        print_dir_content("result after test:", &self.given_dir);
        if let Ok(false) = dir_diff::is_different(&self.expected_dir, &self.given_dir) {
            Ok(())
        } else {
            Err(Error::ExpectedDirNotSame)
        }
    }
}

fn print_dir_content<P: AsRef<Path>>(msg: &str, root: P) {
    info!("{}", &msg);
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        info!("path: {}", &path.display());
        if path.is_file() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            info!("file content: {:#?}\n", &content);
        }
    }
}

impl Files {
    fn boxed(a: &Path, b: &Path) -> Box<dyn Tester> {
        Box::new(Files {
            expected_dir: a.to_path_buf(),
            given_dir: b.to_path_buf(),
        })
    }
}

pub async fn run(assignment: &Assignment, code: &Base64) -> Result<(), Error> {
    let dir_to_test = tempfile::tempdir()?;
    let dir_solution = tempfile::tempdir()?;

    let script_test_path = new_tmp_script_file(assignment.script_type, code)
        .map_err(Error::CantCreatTempFile)?
        .into_temp_path();
    //fs::copy(&assignment.solution_path, &script_solution_path)?;
    cp_include_into(&assignment.include_files, &dir_solution, &dir_to_test)?;

    let test_output = assignment
        .script_type
        .run(&script_test_path, &dir_to_test.path(), &assignment.args)
        .await?;
    info!("running task: {}", &assignment.name);
    let solution_output = assignment
        .script_type
        .run(
            &assignment.solution_path,
            &dir_solution.path(),
            &assignment.args,
        )
        .await?;
    let mut tests: Vec<Box<dyn Tester>> = Vec::new();

    tests.push(Stdout::boxed(solution_output.stdout, test_output.stdout));
    if assignment.check_files {
        tests.push(Files::boxed(&dir_solution.path(), &dir_to_test.path()));
    }
    tests
        .iter()
        .map(|item| item.test())
        .collect::<Result<_, _>>()
}

#[derive(Debug, err_derive::Error, derive_more::From)]
pub enum Error {
    #[error(display = "Time out reached! Script took more than {}.", _1)]
    Timeout(tokio::time::Elapsed, DurationDisplay),
    #[from]
    #[error(display = "Script produced invalid UFT8.")]
    NoUTF8(std::string::FromUtf8Error),
    #[error(display = "Does not contains expected output. {}", _0)]
    WrongOutput(String),
    #[error(display = "Solution dir and tested dir have not the same content")]
    ExpectedDirNotSame,
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
