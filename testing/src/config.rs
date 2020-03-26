use crate::config;
//use std::path::PathBuf;

fn default_port() -> u16 {
    50051
}

fn default_max_curr_test() -> usize {
    if cfg!(target_family = "unix") {
        8
    } else {
        4
    }
}

fn default_image_name() -> String {
    let name = if cfg!(target_family = "unix") {
        "dominicwrege/depp-project-ubuntu:latest"
    } else {
        "mcr.microsoft.com/powershell:latest"
    };
    String::from(name)
}

fn default_timout_secs() -> u64 {
    if cfg!(target_family = "unix") {
        120
    } else {
        180
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_max_curr_test", rename = "max_curr")]
    pub max_curr_test: usize,
    #[serde(default = "default_image_name")]
    pub docker_image: String,
    #[serde(default = "default_timout_secs")]
    pub timeout: u64,
}
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
