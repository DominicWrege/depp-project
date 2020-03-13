use crate::api::{EndPointStatus, IliasId};
use crate::handlers::auth::Credentials;
use crate::rpc_conf::{RpcConfig, RpcEnvConfig};
use deadpool_postgres::Pool;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::RwLock;
#[derive(Clone)]
pub struct State {
    pub inner: Arc<InnerState>,
}

pub struct InnerState {
    pub pending_results: dashmap::DashMap<IliasId, grpc_api::AssignmentResult>,
    pub rpc_conf: RpcConfig,
    pub credentials: Credentials,
    pub to_test_assignments: RwLock<HashSet<IliasId>>,
    pub db_pool: Pool,
}

impl State {
    pub fn new(rpc_conf: RpcEnvConfig, credentials: Credentials, db_pool: Pool) -> State {
        State {
            inner: Arc::new(InnerState {
                pending_results: dashmap::DashMap::new(),
                rpc_conf: rpc_conf.into(),
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
