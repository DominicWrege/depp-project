use serde::Deserialize;
use std::fs;

use err_derive::Error;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub name: String,
    pub assignment: Vec<Assignment>,
}
#[derive(
    Debug, Clone, Hash, Eq, PartialEq, Deserialize, serde::Serialize, Copy, derive_more::From,
)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentId(pub u64);

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Assignment {
    #[serde(default)]
    pub name: String,
    /*pub script_path: PathBuf,*/
    pub output: Option<Pattern>,
    #[serde(default)]
    #[serde(rename = "type")]
    pub commandline: Script,
    #[serde(default)]
    pub args: Vec<String>,
    pub script_contains: Option<Pattern>,
    #[serde(default)]
    pub files: Vec<File>,
}

impl From<usize> for AssignmentId {
    fn from(n: usize) -> Self {
        AssignmentId(u64::try_from(n).unwrap_or_default())
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct File {
    pub path: PathBuf,
    pub content: Option<String>,
}
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Pattern {
    #[serde(default)]
    pub regex: bool,
    pub text: String,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum Script {
    Powershell,
    Batch,
    Python3,
    Shell,
    Bash,
    Awk,
    Sed
}
#[cfg(target_os = "linux")]
impl Script {
    pub fn commandline(self) -> (&'static str, Vec<PathBuf>) {
        match self {
            Script::Powershell => ("pwsh", vec![]),
            Script::Shell => ("sh", vec![]),
            Script::Batch => ("wine", vec!["cmd.exe".into(), "/C".into()]),
            Script::Python3 => ("python3", vec![]),
            Script::Bash => ("bash", vec![]),
            Script::Awk => ("awk", vec![]),
            Script::Sed => ("sed", vec![])
        }
    }
}

#[cfg(target_os = "windows")]
impl Script {
    pub fn commandline(self) -> (&'static str, Vec<PathBuf>) {
        match self {
            Script::Powershell => ("powershell.exe", vec![]),
            Script::Shell => ("sh", vec![]),
            Script::Batch => ("cmd.exe", vec!["/C".into()]),
            Script::Python3 => ("python3", vec![]),
            Script::Bash => ("bash", vec![]),
            Script::Awk => ("awk", vec![]),
            Script::Sed => ("sed", vec![])
        }
    }
}

impl Default for Script {
    fn default() -> Self {
        Script::Batch
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "can`t read config file, {}", _0)]
    ConfigFile(std::io::Error),
    #[error(display = "wrong toml format, {}", _0)]
    Toml(toml::de::Error),
    #[error(display = "given file was not found, {}", _0)]
    IO(std::io::Error),
    #[error(display = "wrong regex was, {}", _0)]
    Regex(regex::Error),
}

pub fn parse_config(path: &Path) -> Result<HashMap<AssignmentId, Assignment>, Error> {
    let file_content = fs::read_to_string(path).map_err(Error::ConfigFile)?;
    let exercise = toml::from_str(&file_content).map_err(Error::Toml)?;
    Ok(into_config_map(exercise))
}

fn into_config_map(conf: Config) -> HashMap<AssignmentId, Assignment> {
    conf.assignment
        .into_iter()
        .enumerate()
        .map(|(id, assignment)| (AssignmentId::from(id), assignment))
        .collect::<HashMap<AssignmentId, Assignment>>()
}
