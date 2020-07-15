//! Checks the script meets certain criterions.
use crate::docker_api::ScriptOutput;
use crate::error::Error;
use crate::error::IOError;
use crate::fs_util;
use async_trait::async_trait;
use futures::pin_mut;
use futures::StreamExt;
use grpc_api::Script;
use grpc_api::{RegexMode, SortStdoutBy};
use log::info;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Checks if the script is correct and if not each function has to return an error.
#[async_trait]
pub trait Checker: Sync + Send {
    async fn check(&self) -> Result<(), Error>;
}

/// Check if the script has created certain files/folders.
#[derive(Debug)]
pub struct FilesChecker {
    expected_dir: PathBuf,
    given_dir: PathBuf,
}

impl FilesChecker {
    pub fn boxed(a: PathBuf, b: PathBuf) -> Box<dyn Checker> {
        Box::new(FilesChecker {
            expected_dir: a,
            given_dir: b,
        })
    }
    fn cmp_file_type(&self, a: &Path, b: &Path) -> bool {
        (a.is_file() && b.is_file()) || (a.is_dir() && b.is_dir())
    }
}
/// Check script or stdout for certain pattern.
#[derive(Debug)]
pub struct RegexChecker {
    regex: Option<Result<regex::Regex, regex::Error>>,
    /// Check stdout or the script content
    mode: RegexMode,
    /// Stdout to check
    stdout: String,
    /// The content of the script to check
    script_content: String,
}
/// initialize struct
impl RegexChecker {
    pub fn boxed<T: AsRef<str>>(
        regex: Option<T>,
        mode: RegexMode,
        stdout: T,
        script_content: &String,
    ) -> Box<dyn Checker> {
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
impl Checker for RegexChecker {
    async fn check(&self) -> Result<(), Error> {
        log::info!("checking with regex");
        if let Some(regex) = self.regex.clone() {
            let regex = regex.map_err(|er| Error::InvalidRegex(er.to_string()))?;
            let res = match self.mode {
                RegexMode::Stdout => (regex.is_match(&self.stdout), self.stdout.clone()),
                RegexMode::ScriptContent => (
                    regex.is_match(&self.script_content),
                    self.script_content.clone(),
                ),
                _ => (true, "".into()),
            };
            return if res.0 {
                Ok(())
            } else {
                Err(Error::NoRegexMatch(res.1, regex.clone()))
            };
        } else {
            Ok(())
        }
    }
}
/// Check stdout with the expected output form the solution.
#[derive(Debug)]
pub struct StdoutChecker {
    expected: String,
    tested: String,
}
/// initialize struct
impl StdoutChecker {
    pub fn boxed(expected: &str, tested: &str) -> Box<dyn Checker> {
        Box::new(StdoutChecker {
            expected: String::from(expected),
            tested: String::from(tested),
        })
    }
}
#[async_trait]
impl Checker for StdoutChecker {
    async fn check(&self) -> Result<(), Error> {
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

#[async_trait]
impl Checker for FilesChecker {
    async fn check(&self) -> Result<(), Error> {
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
                    let solution_content = trim_lines(
                        &fs::read_to_string(&solution_entry)
                            .await
                            .map_err(|e| IOError::ReadFile(e))?,
                    );
                    let result_content = trim_lines(
                        &fs::read_to_string(&path_to_check)
                            .await
                            .map_err(|e| IOError::ReadFile(e))?,
                    );
                    if solution_content != result_content {
                        return Err(Error::ExpectedFileNotSame(
                            solution_entry,
                            solution_content,
                            result_content,
                        ));
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
/// Run the custom script to check if the script is right.
/// $1=Stdout from tested script, $2=script content.
#[derive(Debug)]
pub struct CustomScriptChecker {
    custom_script_content: String,
    tested_script_content: String,
    tested_out: ScriptOutput,
    working_dir: PathBuf,
}
/// initialize struct
impl CustomScriptChecker {
    pub fn boxed(
        c_script_content: &str,
        tested_script_content: &str,
        tested_out: &ScriptOutput,
        cur_dir: &Path,
    ) -> Box<dyn Checker> {
        Box::new(CustomScriptChecker {
            custom_script_content: String::from(c_script_content),
            tested_script_content: String::from(tested_script_content),
            tested_out: tested_out.clone(),
            working_dir: cur_dir.to_path_buf(),
        })
    }
}

#[async_trait]
impl Checker for CustomScriptChecker {
    async fn check(&self) -> Result<(), Error> {
        log::info!("running Custom script");
        let script_type = if cfg!(target_family = "unix") {
            Script::Bash
        } else {
            Script::PowerShell
        };
        let file_custom_script =
            fs_util::new_tmp_script_file(script_type, &self.custom_script_content)
                .map_err(|e| IOError::CreateFile(e))?
                .into_temp_path();

        let prog = if cfg!(target_family = "unix") {
            "bash"
        } else {
            "powershell.exe"
        };
        use tokio::process::Command;
        let outpout = Command::new(prog)
            .arg(&file_custom_script)
            .args(&[&self.tested_out.stdout, &self.tested_script_content])
            .current_dir(&self.working_dir)
            .output()
            .await
            .map_err(|e| IOError::FailedRunCustomScript(e))?;
        if outpout.stderr.is_empty() && outpout.status.success() {
            Ok(())
        } else {
            Err(Error::CustomScript(
                String::from_utf8(outpout.stderr).unwrap_or_default(),
            ))
        }
    }
}
/// Check if the stdout ist sorted asc or desc.
#[derive(Debug)]
pub struct SortedChecker {
    content: String,
    sort_stdout_by: SortStdoutBy,
}
/// initialize struct
impl SortedChecker {
    pub fn boxed(content: &str, sort_stdout_by: SortStdoutBy) -> Box<dyn Checker> {
        Box::new(Self {
            content: content.into(),
            sort_stdout_by,
        })
    }
}

#[async_trait]
impl Checker for SortedChecker {
    async fn check(&self) -> Result<(), Error> {
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
