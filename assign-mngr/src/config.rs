//! Configuration for the admin user provided by the environment prefixed by "DEPP_WEB_".  
//! The admin user logs in with a password and no username required.
use serde::{Deserialize, Deserializer};
use sha2::Digest;
/// Default password: `secret1`
pub fn default_pwd() -> Vec<u8> {
    hash("secret1".into())
}
/// Default port: `5000`
pub fn default_port() -> u16 {
    5000
}
/// Hash wrapper
pub fn hash(s: String) -> Vec<u8> {
    sha2::Sha256::digest(s.as_ref()).to_vec()
}

pub fn has_password<'de, D>(deserial: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserial).unwrap_or_default();
    Ok(hash(s))
}

/// Server config from the environment ```prefix DEPP_WEB_```
#[derive(serde::Deserialize, Debug)]
pub struct ConfigEnv {
    #[serde(deserialize_with = "has_password", default = "default_pwd")]
    pub password: Vec<u8>,
    #[serde(default = "default_port")]
    pub port: u16,
}
/// Configuration for the cookie.
#[derive(Clone, Debug)]
pub struct CookieConfig {
    secure: bool,
    key: [u8; 32],
}

impl CookieConfig {
    pub fn new() -> Self {
        if cfg!(debug_assertions) {
            Self {
                secure: false,
                key: [0u8; 32],
            }
        } else {
            Self {
                secure: true,
                key: rand::random(),
            }
        }
    }
    pub fn secure(&self) -> bool {
        self.secure
    }
    pub fn key(&self) -> [u8; 32] {
        self.key
    }
}

/// Get config from the environment.
pub fn get() -> ConfigEnv {
    match envy::prefixed("DEPP_WEB_").from_env::<ConfigEnv>() {
        Ok(conf) => conf,
        Err(e) => panic!("Wrong env vars: {}", e),
    }
}
