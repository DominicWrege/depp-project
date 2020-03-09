mod config;
mod crash_test;
mod docker_api;
mod fs_util;
mod script;
#[cfg(target_family = "unix")]
use crate::docker_api::{pull_image, LINUX_IMAGE};
#[cfg(target_family = "windows")]
use crate::docker_api::{pull_image, LINUX_IMAGE, MS_IMAGE};
use grpc_api::test_server::{Test, TestServer};
use grpc_api::{AssignmentMsg, AssignmentResult};
use structopt::StructOpt;
use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug)]
pub struct Tester {
    docker: bollard::Docker,
}

impl Tester {
    fn new(docker: bollard::Docker) -> Self {
        Tester { docker }
    }
}

#[tonic::async_trait]
impl Test for Tester {
    async fn run_test(
        &self,
        request: Request<AssignmentMsg>,
    ) -> Result<Response<AssignmentResult>, Status> {
        let msg = request.into_inner();
        // Eror handling when no valid uuid
        let req = request.into_inner();
        if let Some(assignment) = req.assignment {
            let reply = match crash_test::run(&assignment, &req.code_to_test, &self.docker).await {
                Err(crash_test::Error::CantCreatTempFile(e)) | Err(crash_test::Error::Copy(e)) => {
                    log::error!(
                        "Error while creating a tempfile or copying files. The server has to stop."
                    );
                    panic!("{}", e);
                }
                Err(crash_test::Error::Docker(e)) => {
                    log::error!("Some error with the docker API.The server has to stop.");
                    panic!("{}", e);
                }
                Err(e) => AssignmentResult {
                    passed: false,
                    message: Some(e.to_string()),
                    mark: None,
                },
                Ok(_) => AssignmentResult {
                    passed: true,
                    message: None,
                    mark: None,
                },
            };
            Ok(Response::new(reply))
        } else {
            Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "assignmentId was not found",
            ))
        }
    }
}

fn default_port() -> u16 {
    50051
}

#[derive(serde::Deserialize, Debug)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    let opt = Opt::from_args();
    log::info!("Recker instance");
    let docker =
        bollard::Docker::connect_with_local_defaults().expect("Can't connect to docker api.");
    log::info!("Pulling image. This takes some time...");

    #[cfg(target_family = "windows")]
    {
        pull_image(LINUX_IMAGE, &docker).await;
        pull_image(MS_IMAGE, &docker).await;
    }
    #[cfg(target_family = "unix")]
    {
        pull_image(LINUX_IMAGE, &docker).await;
    }

    log::info!("Pulling image done.");
    let test = Tester::new(docker);
    let port = envy::from_env::<ServerConfig>()?.port;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    log::info!("Tester listening on {}", &addr);
    Server::builder()
        .add_service(TestServer::new(test))
        .serve(addr)
        .await?;
    Ok(())
}
