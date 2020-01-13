use crate::api::{AssignmentId, IliasId};
use crate::deep_project;
use crate::deep_project::test_client::TestClient;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

#[derive(Clone)]
pub struct State {
    pub inner: Arc<InnerState>,
}

impl State {
    pub fn new() -> State {
        State {
            inner: Arc::new(InnerState {
                pending_results: dashmap::DashMap::new(),
            }),
        }
    }
}

impl Default for EndPointStatus {
    fn default() -> Self {
        EndPointStatus::Online
    }
}

pub async fn get_rpc_status() -> EndPointStatus {
    match TestClient::connect("http://[::1]:50051").await {
        Ok(_) => EndPointStatus::Online,
        Err(_) => EndPointStatus::Offline,
    }
}

#[derive(serde::Serialize, Debug, Clone, derive_more::Constructor)]
#[serde(rename_all = "camelCase")]
pub struct Meta<'a> {
    pub version: &'static str,
    pub status: &'a EndPointStatus,
}

#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum EndPointStatus {
    Online,
    // Maintenance,
    Offline,
}
pub struct InnerState {
    pub pending_results: dashmap::DashMap<IliasId, deep_project::AssignmentResult>,
}

impl Deref for State {
    type Target = Arc<InnerState>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for State {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
