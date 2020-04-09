use crate::docker_api::ScriptOutput;
use crate::error::Error;
use crate::fs_util;
use async_trait::async_trait;
use futures::pin_mut;
use futures::StreamExt;
use grpc_api::{RegexMode, SortStdoutBy};
use log::info;
use std::path::{Path, PathBuf};
use tokio::fs;

#[async_trait]
pub trait CrashTester: Sync + Send {
    async fn test(&self) -> Result<(), Error>;
}

pub struct FilesChecker {
    expected_dir: PathBuf,
    given_dir: PathBuf,
}

impl FilesChecker {
    pub fn boxed(a: PathBuf, b: PathBuf) -> Box<dyn CrashTester> {
        Box::new(FilesChecker {
            expected_dir: a,
            given_dir: b,
        })
    }
    fn cmp_file_type(&self, a: &Path, b: &Path) -> bool {
        (a.is_file() && b.is_file()) || (a.is_dir() && b.is_dir())
    }
}

pub struct RegexChecker {
    regex: Option<Result<regex::Regex, regex::Error>>,
    mode: RegexMode,
    stdout: String,
    script_content: String,
}

impl RegexChecker {
    pub fn boxed<T: AsRef<str>>(
        regex: Option<T>,
        mode: RegexMode,
        stdout: T,
        script_content: &String,
    ) -> Box<dyn CrashTester> {
        Box::new(RegexChecker {
            regex: regex.and_then(|reg_str| {
                Some(
                    regex::RegexBuilder::new(reg_str.as_ref())
                        .multi_line(true)
                        .build(),
                )
            }),
            mode,
            stdout: stdout.as_ref().to_string(),
            script_content: script_content.into(),
        })
    }
}

#[async_trait]
impl CrashTester for RegexChecker {
    async fn test(&self) -> Result<(), Error> {
        if let Some(regex) = self.regex.clone() {
            let regex = regex.map_err(|er| Error::WrongRegex(er))?;

            if self.mode == RegexMode::Stdout && regex.is_match(&self.stdout) {
                return Err(Error::NoRegexMatch(self.stdout.clone(), regex.clone()));
            } else if self.mode == RegexMode::ScriptContent && regex.is_match(&self.script_content)
            {
                return Err(Error::NoRegexMatch(
                    self.script_content.clone(),
                    regex.clone(),
                ));
            };
        }

        Ok(())
    }
}

pub struct StdoutChecker {
    expected: String,
    tested: String,
}

impl StdoutChecker {
    pub fn boxed(expected: &str, tested: &str) -> Box<dyn CrashTester> {
        Box::new(StdoutChecker {
            expected: String::from(expected),
            tested: String::from(tested),
        })
    }
}

#[async_trait]
impl CrashTester for StdoutChecker {
    async fn test(&self) -> Result<(), Error> {
        log::info!("result stdout: {:#?}", self.tested);
        log::info!("expected stdout: {:#?}", self.expected);
        if self.expected == self.tested {
            Ok(())
        } else {
            Err(Error::WrongOutput(format!(
                "expected stdout:({:#?}) result stdout:({:#?})",
                self.expected, self.tested
            )))
        }
    }
}

// TODO show in message which file in different!!
#[async_trait]
impl CrashTester for FilesChecker {
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
                    let solution_content = trim_lines(&fs::read_to_string(&solution_entry).await?);
                    let result_content = trim_lines(&fs::read_to_string(&path_to_check).await?);
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

pub struct CustomScriptChecker {
    script_content: String,
    expected_output: String,
    solution_output: String,
}

pub struct SortedChecker {
    content: String,
    sort_stdout_by: SortStdoutBy,
}

impl SortedChecker {
    pub fn boxed(content: &str, sort_stdout_by: SortStdoutBy) -> Box<dyn CrashTester> {
        Box::new(Self {
            content: content.into(),
            sort_stdout_by,
        })
    }
}

#[async_trait]
impl CrashTester for SortedChecker {
    async fn test(&self) -> Result<(), Error> {
        use compare::{natural, Compare};
        log::info!("Checking if stdout is sorted.");

        let mut lines = self.content.lines().collect::<Vec<_>>();

        if self.sort_stdout_by == SortStdoutBy::Asc {
            lines.sort_by(|a, b| natural().compare(a, b));
        } else {
            lines.sort_by(|a, b| natural().rev().compare(a, b));
        }
        if lines.join("\n") == self.content {
            Ok(())
        } else {
            Err(Error::NoSorted(self.content.clone()))
        }
    }
}

pub fn trim_lines(s: &str) -> String {
    let ret = s
        .chars()
        .filter(|&c| c != '\r')
        .collect::<String>()
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect::<Vec<String>>();
    ret.join("\n")
}
