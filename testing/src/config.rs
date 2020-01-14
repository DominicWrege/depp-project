use grpc_api::Script;
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub type AssignmentsMap = HashMap<AssignmentId, Assignment>;

#[derive(
    Debug, Clone, Hash, Eq, PartialEq, Deserialize, serde::Serialize, Copy, derive_more::From,
)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentId(pub Uuid);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub name: String,
    pub assignments: HashMap<Uuid, Assignment>,
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
    #[serde(rename = "type")]
    pub script_type: Script,
    #[serde(default)]
    pub args: Vec<String>,
    //pub script_contains: Option<Pattern>, //
}

fn into_absolute_path<'de, D>(deserial: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let relative_path = PathBuf::deserialize(deserial)?;

    fs::canonicalize(relative_path).map_err(de::Error::custom)
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

pub fn parse_config(path: &Path) -> Result<AssignmentsMap, Error> {
    let file_content = fs::read_to_string(path)?;
    let conf: Config = toml::from_str(&file_content)?;
    Ok(into_to_assignments_map(conf.assignments))
}

pub fn into_to_assignments_map(am: HashMap<Uuid, Assignment>) -> AssignmentsMap {
    am.into_iter()
        .map(|(id, assignment)| {
            for path in &assignment.include_files {
                path_exists_and_is_file(&path);
            }
            path_exists_and_is_file(&assignment.solution_path);
            (id.into(), assignment)
        })
        .collect::<_>()
}

fn path_exists_and_is_file(p: &Path) {
    if !p.exists() || p.is_dir() {
        panic!(
            "Config error: path to {:#?} does not exists or is not a file.",
            p
        )
    }
}
