use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs;
use std::path::{Path, PathBuf};

use crate::api::AssignmentId;
use crate::script::Script;
use serde::{de, Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub name: String,
    pub assignment: Vec<Assignment>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Assignment {
    pub name: String,
    #[serde(deserialize_with = "into_absolute_path")]
    pub solution_path: PathBuf,
    #[serde(default)]
    pub include_files: Vec<PathBuf>,
    #[serde(default)]
    pub check_files: bool,
    #[serde(default)]
    #[serde(rename = "type")]
    pub script_type: Script,
    #[serde(default)]
    pub args: Vec<String>,
    pub script_contains: Option<Pattern>, // delete me
}

fn into_absolute_path<'de, D>(deserial: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let relative_path = PathBuf::deserialize(deserial)?;

    fs::canonicalize(relative_path).map_err(de::Error::custom)
}

impl From<usize> for AssignmentId {
    fn from(n: usize) -> Self {
        AssignmentId(u64::try_from(n).unwrap_or_default())
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Pattern {
    #[serde(default)]
    pub regex: bool,
    pub text: String,
}

#[derive(Debug, err_derive::Error, derive_more::From)]
pub enum Error {
    #[from]
    #[error(display = "can`t find config file, {}", _0)]
    ConfigFile(std::io::Error),
    #[from]
    #[error(display = "wrong toml format, {}", _0)]
    Toml(toml::de::Error),
}

pub fn parse_config(path: &Path) -> Result<HashMap<AssignmentId, Assignment>, Error> {
    let file_content = fs::read_to_string(path)?;
    let exercise = toml::from_str(&file_content)?;
    Ok(into_config_map(exercise))
}

fn into_config_map(conf: Config) -> HashMap<AssignmentId, Assignment> {
    conf.assignment
        .into_iter()
        .enumerate()
        .map(|(id, assignment)| {
            for path in &assignment.include_files {
                path_exists_and_is_file(&path);
            }
            path_exists_and_is_file(&assignment.solution_path);
            (AssignmentId::from(id), assignment)
        })
        .collect::<HashMap<AssignmentId, Assignment>>()
}

fn path_exists_and_is_file(p: &Path) {
    if !p.exists() || p.is_dir() {
        panic!(
            "Config error: path to {:#?} does not exists or is not a file.",
            p
        )
    }
}
