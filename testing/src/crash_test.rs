use std::fmt;
use std::path::{Path, PathBuf};
use std::time;

use async_trait::async_trait;
use futures::pin_mut;
use futures::{future, StreamExt};
use log::info;
use tokio::fs;

use crate::fs_util;
use crate::script;
use crate::script::ScriptOutput;
use grpc_api::Assignment;

#[async_trait]
pub trait Tester: Sync + Send {
    async fn test(&self) -> Result<(), Error>;
}
pub struct Stdout {
    expected: ScriptOutput,
    testet: ScriptOutput,
}
pub struct Files {
    expected_dir: PathBuf,
    given_dir: PathBuf,
}

impl Stdout {
    fn boxed(expected: ScriptOutput, testet: ScriptOutput) -> Box<dyn Tester> {
        Box::new(Stdout { expected, testet })
    }
}

#[async_trait]
impl Tester for Stdout {
    async fn test(&self) -> Result<(), Error> {
        let stdout = trim_new_lines(&self.testet.stdout);

        if !self.testet.stderr.is_empty() || self.testet.status_code > 0 {
            //maybe bad syntax
            return Err(Error::ExitCode(self.testet.stderr.clone()));
        }
        let expected_output = trim_new_lines(&self.expected.stdout); // check if solution is also no error
        log::info!("expected stdout: {:#?}", expected_output);
        log::info!("result stdout: {:#?}", stdout);
        if expected_output.contains(&stdout) {
            Ok(())
        } else {
            Err(Error::WrongOutput(format!(
                "expected stdout:({:#?}) result stdout:({:#?})",
                expected_output, stdout
            )))
        }
    }
}

impl Files {
    fn boxed(a: PathBuf, b: PathBuf) -> Box<dyn Tester> {
        Box::new(Files {
            expected_dir: a,
            given_dir: b,
        })
    }
    fn cmp_file_type(&self, a: &Path, b: &Path) -> bool {
        (a.is_file() && b.is_file()) || (a.is_dir() && b.is_dir())
    }
}

#[async_trait]
impl Tester for Files {
    async fn test(&self) -> Result<(), Error> {
        print_dir_content("expected dir:", &self.expected_dir).await?;
        print_dir_content("dir after test:", &self.given_dir).await?;
        let stream = fs_util::ls_dir_content(self.expected_dir.clone());
        pin_mut!(stream);
        while let Some(Ok(solution_entry)) = stream.next().await {
            let path_to_check = &self.given_dir.as_path().join(
                solution_entry.strip_prefix(&self.expected_dir).unwrap(), // TODO err handling
            );
            if path_to_check.exists()
                && self.cmp_file_type(&solution_entry, &path_to_check.as_path())
            {
                if solution_entry.is_file() {
                    let solution_content =
                        trim_new_lines(&fs::read_to_string(&solution_entry).await?);
                    let result_content = trim_new_lines(&fs::read_to_string(&path_to_check).await?);
                    if solution_content != result_content {
                        return Err(Error::ExpectedFileNotSame(solution_content, result_content));
                    }
                }
            } else {
                return Err(Error::ExpectedDirNotSame);
            }
        }

        Ok(())
    }
}

async fn print_dir_content(msg: &str, root: &Path) -> Result<(), Error> {
    info!("{}", &msg);
    let stream = fs_util::ls_dir_content(root.to_path_buf().clone());
    pin_mut!(stream);
    while let Some(Ok(entry)) = stream.next().await {
        info!("    path: {}", &entry.display());
        if entry.is_file() {
            let content = fs::read_to_string(&entry).await.unwrap_or_default();
            info!("    file content: {:#?}", &content);
        }
    }
    Ok(())
}

pub async fn run(
    assignment: &Assignment,
    code: &str,
    docker: &bollard::Docker,
) -> Result<(), Error> {
    let context_dir = fs_util::copy_items_include(&assignment.include_files).await?;
    let script_test_path = fs_util::new_tmp_script_file(assignment.script_type.into(), code)
        .map_err(Error::CantCreatTempFile)?
        .into_temp_path();
    let script_solution_path =
        fs_util::new_tmp_script_file(assignment.script_type.into(), &assignment.solution)
            .map_err(Error::CantCreatTempFile)?
            .into_temp_path();
    info!("running task: {}", &assignment.name);
    let test_output = script::run_in_container(
        &docker,
        &assignment.script_type.into(),
        &script_test_path,
        &context_dir.path(),
        &assignment.args,
    )
    .await?;
    let solution_context_dir = fs_util::copy_items_include(&assignment.include_files).await?;
    let solution_output = script::run_in_container(
        &docker,
        &assignment.script_type.into(),
        &script_solution_path,
        &solution_context_dir.path(),
        &assignment.args,
    )
    .await?;
    let mut tests: Vec<Box<dyn Tester>> = Vec::new();

    tests.push(Stdout::boxed(solution_output, test_output));
    tests.push(Files::boxed(
        solution_context_dir.path().to_path_buf(),
        context_dir.path().to_path_buf(),
    ));
    let _ = future::try_join_all(tests.iter().map(|item| async move { item.test().await })).await?;
    Ok(())
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
    #[error(display = "Wrong file content: expected({:#?}) result({:#?})", _0, _1)]
    ExpectedFileNotSame(String, String),
    #[error(display = "Can't create temp file. {}", _0)]
    CantCreatTempFile(std::io::Error),
    #[from]
    #[error(display = "Could not copy included files for testing {}", _0)]
    Copy(std::io::Error),
    #[error(display = "IO error while reading the dir {:?}", _0)]
    ListDir(PathBuf),
    #[error(display = "Docker error {}", _0)]
    Docker(String),
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
