use std::path::Path;

mod config;
mod crash_test;
mod fs_util;
mod script;

use grpc_api::test_server::{Test, TestServer};
use grpc_api::{
    AssignmentIdRequest, AssignmentIdResponse, AssignmentMsg, AssignmentResult, VecAssignmentsShort,
};
use structopt::StructOpt;
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;
//use base64;
use config::{parse_config, AssignmentsMap};

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long)]
    config: std::path::PathBuf,
}

#[derive(Default, Debug)]
pub struct Tester {
    assignments: AssignmentsMap,
}

impl Tester {
    fn new(assignments: AssignmentsMap) -> Self {
        Tester { assignments }
    }
}

#[tonic::async_trait]
impl Test for Tester {
    async fn run_test(
        &self,
        request: Request<AssignmentMsg>,
    ) -> Result<Response<AssignmentResult>, Status> {
        let msg = request.into_inner();
        let (assignment, code) = (msg.assignment, msg.source_code);
        if let Some(assignment) = assignment {
            let reply = match crash_test::run(&config::Assignment::from(assignment), &code).await {
                Err(crash_test::Error::CantCreatTempFile(e)) | Err(crash_test::Error::Copy(e)) => {
                    //wait_print_err(e).await;
                    panic!(e);
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
                "assignment is null",
            ))
        }
    }

    async fn get_assignments(
        &self,
        _: Request<()>,
    ) -> Result<Response<VecAssignmentsShort>, Status> {
        let reply = assignments_to_msg(self.assignments.clone());
        Ok(Response::new(reply))
    }
    async fn assignment_exists(
        &self,
        request: Request<AssignmentIdRequest>,
    ) -> Result<Response<AssignmentIdResponse>, Status> {
        //TODO fix unwrap
        let id = Uuid::parse_str(&request.into_inner().assignment_id).unwrap();
        let ret = self.assignments.get(&id).map(|x| x.clone()).is_some();

        Ok(Response::new(AssignmentIdResponse { found: ret }))
    }

    async fn get_assignment(
        &self,
        request: Request<AssignmentIdRequest>,
    ) -> Result<Response<grpc_api::Assignment>, Status> {
        //TODO fix unwrap
        let id = Uuid::parse_str(&request.into_inner().assignment_id).unwrap();
        if let Some(assignment) = &self.assignments.get(&id) {
            let ret = grpc_api::Assignment {
                name: assignment.name.clone(),
                solution_path: assignment
                    .solution_path
                    .to_str()
                    .unwrap_or_default()
                    .to_string(),
                include_files: assignment
                    .include_files
                    .iter()
                    .map(|p| p.to_str().unwrap_or_default().to_string())
                    .collect::<Vec<_>>(),
                script_type: assignment.script_type.into(),
                args: assignment.args.clone(),
            };
            Ok(Response::new(ret))
        } else {
            Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "id is not found",
            ))
        }
    }
}

impl From<grpc_api::Assignment> for config::Assignment {
    fn from(assignment: grpc_api::Assignment) -> Self {
        config::Assignment {
            name: assignment.name.clone(),
            solution_path: Path::new(&assignment.solution_path).to_path_buf(),
            include_files: assignment
                .include_files
                .iter()
                .map(|p| Path::new(&p).to_path_buf())
                .collect::<Vec<_>>(),
            script_type: assignment.script_type.into(),
            args: assignment.args.clone(),
        }
    }
}

fn assignments_to_msg(thing: AssignmentsMap) -> VecAssignmentsShort {
    let a = thing
        .into_iter()
        .map(|(id, a)| grpc_api::AssignmentShort {
            name: a.name,
            assignment_id: id.to_string(),
        })
        .collect::<_>();
    VecAssignmentsShort { assignments: a }
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
    let config = parse_config(&opt.config)?;
    log::info!("Exercise: {}", &config.name);
    let test = Tester::new(config.assignments);
    let port = envy::from_env::<ServerConfig>()?.port;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    log::info!("Tester listening on {}", &addr);
    Server::builder()
        .add_service(TestServer::new(test))
        .serve(addr)
        .await?;

    Ok(())
}
