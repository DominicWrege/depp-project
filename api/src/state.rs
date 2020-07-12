//! Maneging the global state accessed by multiple threads.
use crate::api::{EndPointStatus, IliasId};
use crate::handlers::auth::Credentials;
use crate::rpc_conf::{RpcConfig, RpcEnvConfig};
use deadpool_postgres::Pool;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Wrapper for the [InnerState](struct.InnerState.html) which uses [Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html) to have thread save shareable state.
#[derive(Clone)]
pub struct State {
    pub inner: Arc<InnerState>,
}
/// You can call it the real state
pub struct InnerState {
    /// All ready test results waiting to be picked up. Maybe store them in a DB instead of memory?!.
    pub pending_results: dashmap::DashMap<IliasId, grpc_api::AssignmentResult>,
    /// RPC Config so all handlers have access to it.
    pub rpc_conf: RpcConfig,
    ///The HTTP basic access authentication credentials.
    pub credentials: Credentials,
    /// Thread safe hashset of all submissions which are not tested yet.
    pub to_test_assignments: RwLock<HashSet<IliasId>>,
    /// DB connection pool using deadpool
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
