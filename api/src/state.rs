use crate::api::IliasId;
use failure::_core::convert::TryFrom;
use grpc_api::test_client::TestClient;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::convert::TryInto;
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

impl TryFrom<String> for Sha256 {
    type Error = failure::Error;

    fn try_from(s: String) -> Result<Sha256, Self::Error> {
        let pwd = sha2::Sha256::digest(&s.as_bytes()).to_vec();
        Ok(Sha256(pwd))
    }
}

fn default_user() -> String {
    String::from("user")
}
fn default_pwd() -> String {
    String::from("wasd4221")
}

#[derive(Debug, serde::Deserialize)]
pub struct Credentials {
    username: String,
    password: Sha256,
}

#[derive(Debug, serde::Deserialize)]
pub struct CredentialsEnv {
    #[serde(default = "default_user")]
    username: String,
    #[serde(default = "default_pwd")]
    password: String,
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
    match envy::prefixed("API_").from_env::<CredentialsEnv>() {
        Ok(cred) => Credentials {
            username: cred.username,
            password: cred.password.try_into().unwrap(),
        },
        Err(err) => panic!("Bad credentials! err: {}", err),
    }
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
