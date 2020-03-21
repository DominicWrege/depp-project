use crate::base64::Base64;
use grpc_api::AssignmentId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
            0: "default_ilias_id".to_string(),
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
#[derive(Debug, serde::Serialize, derive_more::Constructor)]
#[serde(rename_all = "camelCase")]
pub struct SubmissionExample {
    pub ilias_id: IliasId,
    pub source_code: &'static str,
    pub assignment_id: AssignmentId,
}

// TODO update swagger
#[derive(Serialize, Debug, Clone, derive_more::Constructor)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub version: &'static str,
    pub linux_rpc_status: EndPointStatus,
    pub windows_rpc_status: EndPointStatus,
}
// TODO update swagger

#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum EndPointStatus {
    Online,
    // Maintenance,
    Offline,
}
