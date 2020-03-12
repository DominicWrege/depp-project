use crate::api::{EndPointStatus, IliasId};
use crate::handlers::auth::Credentials;
use deadpool_postgres::Pool;
use grpc_api::{Script, TargetOs};
use serde::Deserialize;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

#[derive(Clone)]
pub struct State {
    pub inner: Arc<InnerState>,
}

fn default_addr() -> Url {
    Url::parse("http://127.0.0.1:50051").unwrap()
}
#[derive(Deserialize, Debug)]
pub struct RpcConfig {
    #[serde(default = "default_addr")]
    linux_rpc_url: Url,
    #[serde(default = "default_addr")]
    ms_rpc_url: Url,
}

impl RpcConfig {
    pub fn url(&self, script_type: &Script) -> String {
        match script_type.target_os() {
            TargetOs::Windows => self.ms_rpc_url.to_string(),
            TargetOs::Unix => self.linux_rpc_url.to_string(),
        }
    }

    pub async fn status(&self) -> EndPointStatus {
        use futures::try_join;
        use grpc_api::test_client::TestClient;
        match try_join!(
            TestClient::connect(self.linux_rpc_url.to_string()),
            TestClient::connect(self.ms_rpc_url.to_string())
        ) {
            Ok(_) => EndPointStatus::Online,
            Err(_) => {
                log::warn!("One or boot RPCs is  offline");
                EndPointStatus::Offline
            }
        }
    }
}

pub struct InnerState {
    pub pending_results: dashmap::DashMap<IliasId, grpc_api::AssignmentResult>,
    pub rpc_conf: RpcConfig,
    pub credentials: Credentials,
    pub to_test_assignments: RwLock<HashSet<IliasId>>,
    pub db_pool: Pool,
}

impl State {
    pub fn new(rpc_conf: RpcConfig, credentials: Credentials, db_pool: Pool) -> State {
        State {
            inner: Arc::new(InnerState {
                pending_results: dashmap::DashMap::new(),
                rpc_conf,
                credentials,
                to_test_assignments: RwLock::new(HashSet::new()),
                db_pool,
            }),
        }
    }
}

impl Default for EndPointStatus {
    fn default() -> Self {
        EndPointStatus::Online
    }
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
