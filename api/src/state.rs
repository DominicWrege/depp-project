use crate::api::IliasId;
use grpc_api::test_client::TestClient;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use url::Url;

#[derive(Clone)]
pub struct State {
    pub inner: Arc<InnerState>,
}

fn default_addr() -> Url {
    Url::parse("http://127.0.0.1:50051").unwrap()
}
#[derive(serde::Deserialize, Debug)]
pub struct RpcConfig {
    #[serde(default = "default_addr")]
    rpc_url: Url,
}

impl State {
    pub fn new(rpc_conf: RpcConfig) -> State {
        State {
            inner: Arc::new(InnerState {
                pending_results: dashmap::DashMap::new(),
                rpc_url: rpc_conf.rpc_url,
            }),
        }
    }
}

impl Default for EndPointStatus {
    fn default() -> Self {
        EndPointStatus::Online
    }
}

pub async fn get_rpc_status(rpc_url: &Url) -> EndPointStatus {
    match TestClient::connect(rpc_url.as_str().to_owned()).await {
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
    pub pending_results: dashmap::DashMap<IliasId, grpc_api::AssignmentResult>,
    pub rpc_url: Url,
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
