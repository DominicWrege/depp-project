//! Implements **HTTP basic access authentication**. Stores the credentials like username and password(encrypted) provided by environment variables in memory.

use crate::handlers::error::Error;
use crate::state::State;
use actix_web::dev::ServiceRequest;
use actix_web::web;
use actix_web_httpauth::extractors::basic::BasicAuth;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::convert::TryFrom;
use std::convert::TryInto;

pub async fn handle_basic_auth(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, actix_web::Error> {
    let state: web::Data<State> = req.app_data().unwrap();
    match credentials.password() {
        Some(cred) => {
            let pwd = sha2::Sha256::digest(cred.as_bytes()).to_vec();
            if credentials.user_id() == state.credentials.username()
                && pwd == state.credentials.password()
            {
                Ok(req)
            } else {
                Err(Error::Unauthorized.into_actix_web_err())
            }
        }
        None => Err(Error::Unauthorized.into_actix_web_err()),
    }
}

/// Default username: ```user```
fn default_user() -> String {
    String::from("user")
}
/// Default password: ```wasd4221```
fn default_pwd() -> String {
    String::from("wasd4221")
}
/// Credentials stored in memory.
#[derive(Debug, serde::Deserialize)]
pub struct Credentials {
    username: String,
    password: Sha256,
}

/// Credentials provided by the environment prefixed by "DEPP_API_".  
/// Example:  
/// ```DEPP_API_USERNAME="tom"```  
/// ```DEPP_API_USERNAME="passwordone"```
#[derive(Debug, serde::Deserialize)]
pub struct CredentialsEnv {
    #[serde(default = "default_user")]
    username: String,
    #[serde(default = "default_pwd")]
    password: String,
}
/// Sha256 Wrapper that supports serialisation.
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
    match envy::prefixed("DEPP_API_").from_env::<CredentialsEnv>() {
        Ok(cred) => Credentials {
            username: cred.username,
            password: cred.password.try_into().unwrap(),
        },
        Err(err) => panic!("Bad credentials! err: {}", err),
    }
}

impl TryFrom<String> for Sha256 {
    type Error = failure::Error;

    fn try_from(s: String) -> Result<Sha256, Self::Error> {
        let pwd = sha2::Sha256::digest(&s.as_bytes()).to_vec();
        Ok(Sha256(pwd))
    }
}
