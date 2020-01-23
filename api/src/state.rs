use crate::api::IliasId;
use grpc_api::test_client::TestClient;
use serde::{Deserialize, Serialize};
use sha2::Digest;
//use std::convert::TryInto;
use std::ops::{Deref, DerefMut};
//use std::path::Path;
use std::sync::Arc;
//use tokio::fs;
use failure::_core::str::FromStr;
use structopt::StructOpt;
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

impl FromStr for Sha256 {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pwd = sha2::Sha256::digest(&s.as_bytes()).to_vec();
        Ok(Sha256(pwd))
    }
}

#[derive(Debug, StructOpt, serde::Deserialize, serde::Serialize)]
pub struct Credentials {
    #[structopt(short, long, default_value = "user")]
    username: String,
    #[structopt(short = "p", long = "password", default_value = "wasd4221")]
    password: Sha256,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sha256(#[serde(with = "hex_serde")] pub Vec<u8>);

impl Credentials {
    pub fn username(&self) -> &str {
        &self.username
    }
    pub fn password(&self) -> Vec<u8> {
        self.password.0.to_vec()
    }
}

pub fn get_credentials() -> Credentials {
    Credentials::from_args()
}

impl State {
    pub fn new(rpc_conf: RpcConfig, credentials: Credentials) -> State {
        State {
            inner: Arc::new(InnerState {
                pending_results: dashmap::DashMap::new(),
                rpc_url: rpc_conf.rpc_url,
                credentials,
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
    pub credentials: Credentials,
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
