//! A ```JSON``` only REST API.
use crate::base64::Base64;
use grpc_api::AssignmentId;
use serde::{Deserialize, Serialize};

/// The short version of an Assigment with only ```id``` and ```name```.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentShort {
    #[serde(rename = "assignmentId")]
    pub id: AssignmentId,
    pub name: String,
}

/// An unique ID for the Ilias people for they usage. Each ```rust IliasId ``` stands for each [Submission](struct.Submission.html)
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
            0: "some_ilias_id".to_string(),
        }
    }
}
/// The Submission uploaded by the Student which a to be tested.

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Submission {
    /// A unique ID crated and used by Ilias.
    pub ilias_id: IliasId,
    /// A Base64 encoded representation for the provided source code.
    pub source_code: Base64,
    /// A unique ID (UUID) crated the postgresql to identify each assignment.
    pub assignment_id: AssignmentId,
}
/// Just a little example which I can return for nice error message if someone forgot hwo to use my api.
#[derive(Debug, serde::Serialize, derive_more::Constructor)]
#[serde(rename_all = "camelCase")]
pub struct SubmissionExample {
    pub ilias_id: IliasId,
    pub source_code: &'static str,
    pub assignment_id: AssignmentId,
}

/// The current status of the booth RPC endpoints.
#[derive(Serialize, Debug, Clone, derive_more::Constructor)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub linux_rpc_status: EndPointStatus,
    pub windows_rpc_status: EndPointStatus,
}

/// The Version of this crate only to be serialized.
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub version: &'static str,
}
/// The current status of the one RPC endpoint.
#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum EndPointStatus {
    Online,
    Offline,
}
