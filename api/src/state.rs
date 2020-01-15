use crate::api::IliasId;
use grpc_api::test_client::TestClient;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::convert::TryInto;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sha256(#[serde(with = "hex_serde")] pub [u8; 32]);

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Credentials {
    username: String,
    password: Sha256,
}

impl Credentials {
    pub fn new(username: String, pwd: Sha256) -> Credentials {
        Credentials {
            username: username,
            password: pwd,
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }
    pub fn password(&self) -> Vec<u8> {
       self.password.0.to_vec()
    }
}

impl Default for Credentials{
    fn default() -> Self {
        let mut haser = sha2::Sha256::new();
        haser.input("wasd");
        let result = haser.result().try_into().unwrap();
        Credentials::new("user".into(), Sha256(result) )
    }
}

pub async fn get_credentials() -> Result<Credentials, tokio::io::Error> {
    let file_path = Path::new("./credentials_api.toml");
    if !file_path.exists() {
        fs::File::create(file_path).await?;
        let cred = Credentials::default();
        fs::write(&file_path, toml::to_string(&cred).unwrap()).await?;
        Ok(cred)
    }else{
        let file_content = fs::read_to_string(file_path).await?;
        let cred = toml::from_str::<Credentials>(&file_content)?;
        Ok(cred)
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
