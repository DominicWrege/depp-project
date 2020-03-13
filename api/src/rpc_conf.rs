use crate::api::EndPointStatus;
use grpc_api::{Script, TargetOs};
use serde::Deserialize;
use std::fmt::Formatter;
use url::Url;

fn default_addr() -> Url {
    Url::parse("http://127.0.0.1:50051").unwrap()
}
#[derive(Deserialize, Debug)]
pub struct RpcEnvConfig {
    #[serde(default = "default_addr")]
    linux_rpc_url: Url,
    #[serde(default = "default_addr")]
    ms_rpc_url: Url,
}
pub struct AllEndpointStatus {
    pub windows: EndPointStatus,
    pub linux: EndPointStatus,
}
#[derive(Clone, Debug)]
pub struct RpcMeta {
    pub rpc_url: Url,
    pub plattform: &'static str,
}

impl RpcMeta {
    fn new(rpc_url: Url, plattform: &'static str) -> Self {
        RpcMeta { rpc_url, plattform }
    }
}

impl From<RpcEnvConfig> for RpcConfig {
    fn from(rpc_config: RpcEnvConfig) -> Self {
        Self {
            windows: RpcMeta::new(rpc_config.ms_rpc_url, "windows"),
            linux: RpcMeta::new(rpc_config.linux_rpc_url, "linux"),
        }
    }
}

impl std::fmt::Display for RpcMeta {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "The {} testing server: {} seems to be not reachable",
            self.plattform, self.rpc_url
        )
    }
}

pub struct RpcConfig {
    windows: RpcMeta,
    linux: RpcMeta,
}

impl RpcConfig {
    pub fn meta(&self, script_type: &Script) -> &RpcMeta {
        match script_type.target_os() {
            TargetOs::Windows => &self.windows,
            TargetOs::Unix => &self.linux,
        }
    }
    pub async fn status(&self) -> AllEndpointStatus {
        use grpc_api::test_client::TestClient;

        let (l, w) = futures::join!(
            TestClient::connect(self.linux.rpc_url.to_string()),
            TestClient::connect(self.windows.rpc_url.to_string())
        );
        AllEndpointStatus {
            windows: endpoint_status(w, &self.windows),
            linux: endpoint_status(l, &self.linux),
        }
    }
}

fn endpoint_status<T, E>(r: Result<T, E>, context: &RpcMeta) -> EndPointStatus {
    if r.is_ok() {
        EndPointStatus::Online
    } else {
        log::warn!(
            "grpc {} {} seems to be offline",
            &context.plattform,
            &context.rpc_url
        );
        EndPointStatus::Offline
    }
}
