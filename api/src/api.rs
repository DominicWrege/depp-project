use crate::base64::Base64;
use grpc_api::{AssignmentId, Script};
use serde::{Deserialize, Serialize};

/*
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
*/

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentShort {
    #[serde(rename = "assignmentId")]
    pub id: AssignmentId,
    pub name: String,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Submission {
    pub ilias_id: IliasId,
    pub source_code: Base64,
    pub assignment_id: AssignmentId,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Mark {
    VeryGood,
    Ok,
    Bad,
}
