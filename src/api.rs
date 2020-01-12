use crate::base64::Base64;
use crate::config::Assignment;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentShort<'a> {
    #[serde(rename = "assignmentId")]
    pub id: AssignmentId,
    pub name: &'a str,
}
impl Assignment {
    pub fn into_short(&self, assignment_id: AssignmentId) -> AssignmentShort<'_> {
        AssignmentShort {
            id: assignment_id,
            name: self.name.as_ref(),
        }
    }
}

#[derive(
    Debug,
    Clone,
    Hash,
    Eq,
    PartialEq,
    Deserialize,
    Serialize,
    Copy,
    derive_more::Display,
    derive_more::From,
)]
#[serde(rename_all = "camelCase")]
#[display(fmt = "{}", _0)]
pub struct IliasId(u64);

#[derive(
    Debug, Clone, Hash, Eq, PartialEq, Deserialize, serde::Serialize, Copy, derive_more::From,
)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentId(pub u64);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Submission {
    pub ilias_id: IliasId,
    pub source_code: Base64,
    pub assignment_id: AssignmentId,
}

#[derive(Debug, Deserialize, Serialize, derive_more::Constructor)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentResult {
    pub passed: bool,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub mark: Option<Mark>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Mark {
    VeryGood,
    Ok,
    Bad,
}
