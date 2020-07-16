//! RPC Config
use crate::api::EndPointStatus;
use grpc_api::{Script, TargetOs};
use serde::Deserialize;
use std::fmt::{Debug, Formatter};
use url::Url;
/*
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};
*/

// DEPP_API_ as prefix
/// Default is: ```http://127.0.0.2:50051```
fn default_addr() -> Url {
    Url::parse("http://192.168.178.98:50051").unwrap()
}
/// Reading the environment variables.
pub fn get_config() -> Result<RpcEnvConfig, envy::Error> {
    envy::prefixed("DEPP_API_").from_env::<RpcEnvConfig>()
}
/// The RPC config via the environment variables using ```DEPP_API_``` as prefix.
#[derive(Deserialize, Debug)]
pub struct RpcEnvConfig {
    #[serde(default = "default_addr")]
    linux_rpc_url: Url,
    #[serde(default = "default_addr")]
    ms_rpc_url: Url,
}
/// Bucket for storing the Windows and Linux RPC status.
pub struct AllEndpointStatus {
    pub windows: EndPointStatus,
    pub linux: EndPointStatus,
}

/// To distinguish which RPC host is on which platform.
#[derive(Clone, Debug)]
pub struct RpcMeta {
    pub rpc_url: Url,
    pub platform: &'static str,
}

impl RpcMeta {
    fn new(rpc_url: Url, plattform: &'static str) -> Self {
        RpcMeta {
            rpc_url,
            platform: plattform,
        }
    }
}

impl From<RpcEnvConfig> for RpcConfig {
    fn from(rpc_config: RpcEnvConfig) -> Self {
        //let cert = std::fs::read_to_string("./rootCA.pem").unwrap();

        Self {
            windows: RpcMeta::new(rpc_config.ms_rpc_url, "windows"),
            linux: RpcMeta::new(rpc_config.linux_rpc_url, "linux"),
            /*            tls_config: ClientTlsConfig::new()
            .ca_certificate(Certificate::from_pem(&cert))
            .domain_name("localhost".to_string()),*/
        }
    }
}

impl std::fmt::Display for RpcMeta {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "The {} testing server: {} seems to be not reachable",
            self.platform, self.rpc_url
        )
    }
}
/// This is the RPC config which uses the RPC client.It's used for the communication between the api and the testing server. Two server are required in order to test all necessary kind of scripts.
/// The Linux server test all the ```Bash, Python, sed...``` scripts.
/// And the Windows is only there to test ```PowerShell``` and ```batch``` scripts natively.
pub struct RpcConfig {
    /// The Windows RPC Host.
    windows: RpcMeta,
    /// The Linux RPC Host.
    linux: RpcMeta,
}

impl RpcConfig {
    /// Decides which script belongs to which platform to test.
    pub fn meta(&self, script_type: &Script) -> &RpcMeta {
        match script_type.target_os() {
            TargetOs::Windows => &self.windows,
            TargetOs::Unix => &self.linux,
        }
    }
    /// The status of the RPC Host.
    pub async fn status(&self) -> AllEndpointStatus {
        use grpc_api::test_client::TestClient;
        dbg!(&self.linux.rpc_url.to_string());
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
            "GRPC {} {} seems to be offline",
            &context.platform,
            &context.rpc_url
        );
        EndPointStatus::Offline
    }
}
