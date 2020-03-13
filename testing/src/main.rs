mod config;
mod crash_test;
mod docker_api;
mod fs_util;
mod grpc_tester;
mod sema_wrap;
use crate::docker_api::{pull_image, TESTING_IMAGE};
use grpc_api::test_server::TestServer;
use tonic::transport::Server;

fn default_port() -> u16 {
    50051
}
#[cfg(target_family = "windows")]
fn default_max_curr_test() -> usize {
    4
}

#[cfg(target_family = "unix")]
fn default_max_curr_test() -> usize {
    8
}

#[derive(serde::Deserialize, Debug)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default = "default_max_curr_test")]
    max_curr_test: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    log::info!("Recker instance");
    let docker =
        bollard::Docker::connect_with_local_defaults().expect("Can't connect to docker api.");
    log::info!("Pulling image. This takes some time...");
    pull_image(TESTING_IMAGE, &docker).await;
    log::info!("Pulling image done.");
    let config = envy::from_env::<ServerConfig>()?;
    log::info!(
        "Limiting test to {} at the same  time.",
        config.max_curr_test
    );
    let test = grpc_tester::Tester::new(docker, config.max_curr_test);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], config.port));
    log::info!("Tester listening on {}", &addr);

    Server::builder()
        .add_service(TestServer::new(test))
        .serve(addr)
        .await?;
    Ok(())
}
