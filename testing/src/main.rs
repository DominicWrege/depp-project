mod config;
mod crash_test;
mod docker_api;
mod fs_util;
mod grpc_tester;
mod sema_wrap;
use crate::docker_api::DockerWrap;
use grpc_api::test_server::TestServer;
//use tonic::transport::{Identity, Server, ServerTlsConfig};
use tonic::transport::Server;

fn main() {
    if let Err(e) = tokio::runtime::Runtime::new().unwrap().block_on(run()) {
        log::error!("{}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), failure::Error> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    // TODO Prefix
    let config = envy::from_env::<config::ServerConfig>()?;
    log::info!("Pulling docker the required image. This may take some time...");
    let docker_api = DockerWrap::new(config.docker_image);
    docker_api.pull_image().await?;
    log::info!("Pulling image done.");
    log::info!(
        "Limiting test to {} at the same  time.",
        config.max_curr_test
    );
    let test = grpc_tester::Tester::new(docker_api, config.max_curr_test);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], config.port));
    /*    let cert = tokio::fs::read(&config.cert_path).await?;
    let key = tokio::fs::read(&config.key_path).await?;
    let identity = Identity::from_pem(cert, key);*/
    log::info!("Tester listening on {}", &addr);
    Server::builder()
        /*        .tls_config(ServerTlsConfig::new().identity(identity))*/
        .add_service(TestServer::new(test))
        .serve(addr)
        .await?;
    Ok(())
}
