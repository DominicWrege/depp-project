use crate::base64::Base64;
use grpc_api::{AssignmentId, Script};
use serde::{Deserialize, Serialize};
//TODO fix me
use serde::{de, Deserializer};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Assignment {
    pub name: String,
    #[serde(deserialize_with = "into_absolute_path")]
    pub solution: PathBuf,
    #[serde(default)]
    pub include_files: Vec<PathBuf>,
    #[serde(default)]
    #[serde(rename = "type")]
    pub script_type: Script,
    #[serde(default)]
    pub args: Vec<String>,
}

fn into_absolute_path<'de, D>(deserial: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let relative_path = PathBuf::deserialize(deserial)?;

    std::fs::canonicalize(relative_path).map_err(de::Error::custom)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentShort<'a> {
    #[serde(rename = "assignmentId")]
    pub id: AssignmentId,
    pub name: &'a str,
}

#[derive(
    Debug,
    Clone,
    Hash,
    Eq,
    PartialEq,
    Deserialize,
    Serialize,
    derive_more::Display,
    derive_more::From,
)]
#[serde(rename_all = "camelCase")]
#[display(fmt = "{}", _0)]
pub struct IliasId(String);

impl Default for IliasId {
    fn default() -> Self {
        Self {
            0: "abcdefid".to_string(),
        }
    }
}

/*#[derive(
    Debug,
    Clone,
    Hash,
    Eq,
    PartialEq,
    Deserialize,
    serde::Serialize,
    Copy,
    derive_more::From,
    derive_more::Display,
)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentId(pub Uuid);*/

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Submission {
    pub ilias_id: IliasId,
    pub source_code: Base64,
    pub assignment_id: AssignmentId,
}

// #[derive(Debug, Deserialize, Serialize)]
// #[serde(rename_all = "camelCase")]
// pub struct AssignmentResult {
//     pub passed: bool,
//     #[serde(default)]
//     pub message: Option<String>,
//     #[serde(default)]
//     pub mark: Option<Mark>,
// }

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Mark {
    VeryGood,
    Ok,
    Bad,
}
