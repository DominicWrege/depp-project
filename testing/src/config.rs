//! The config provided by environment variables.

use crate::config;
/// Default RPC port: `50051`.
fn default_port() -> u16 {
    50051
}
/// Default concurrent test: `Linux: 10, Windows: 5`
fn default_max_curr_test() -> usize {
    if cfg!(target_family = "unix") {
        10
    } else {
        5
    }
}
/// Default Docker image for running the scripts.  
/// Linux: `dominicwrege/depp-project-ubuntu:latest`,  
/// Windows: `mcr.microsoft.com/powershell:latest`
fn default_image_name() -> String {
    let name = if cfg!(target_family = "unix") {
        "dominicwrege/depp-project-ubuntu:latest"
    } else {
        "mcr.microsoft.com/powershell:latest"
    };
    String::from(name)
}
/// Default timeout each test in sec.
/// `Linux: 120sec, Windows: 180sec`                                             
fn default_timout_secs() -> u64 {
    if cfg!(target_family = "unix") {
        120
    } else {
        180
    }
}
/// The config for setting up the server
#[derive(serde::Deserialize, Debug)]
pub struct ServerConfig {
    /// Define RPC port on which will the server listen. The default port is `50051`.
    #[serde(default = "default_port")]
    pub port: u16,
    /// Limit the concurrent running tests.
    #[serde(default = "default_max_curr_test", rename = "max_curr")]
    pub max_curr_test: usize,
    #[serde(default = "default_image_name")]
    /// The Docker for running the script inside.
    pub docker_image: String,
    #[serde(default = "default_timout_secs")]
    /// The timeout in seconds for each test.
    pub timeout: u64,
}
/// Get config from the environment.
pub fn get_config() -> Result<ServerConfig, envy::Error> {
    envy::prefixed("DEPP_TEST_").from_env::<config::ServerConfig>()
}

/*
fn default_key_path() -> PathBuf {
    PathBuf::from("./localhost-key.pem")
}
fn default_cert_path() -> PathBuf {
    PathBuf::from("./localhost.pem")
}
*/
