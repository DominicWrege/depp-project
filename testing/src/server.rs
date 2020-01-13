use std::collections::HashMap;
use std::path::Path;

mod config;
mod crash_test;
mod fs_util;
mod script;

use deep_project::test_server::{Test, TestServer};
use deep_project::{
    AssignmentIdRequest, AssignmentIdResponse, AssignmentMsg, AssignmentResult, VecAssignmentsShort,
};
use structopt::StructOpt;
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;
//use base64;
use config::{parse_config, AssignmentId};

pub mod deep_project {
    tonic::include_proto!("deep_project");
}

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long)]
    config: std::path::PathBuf,
}

#[derive(Default, Debug)]
pub struct Tester {
    assignments: HashMap<AssignmentId, config::Assignment>,
}

impl Tester {
    fn new(assignments: HashMap<AssignmentId, config::Assignment>) -> Self {
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
        let reply = self.assignments.clone().into();
        Ok(Response::new(reply))
    }
    async fn assignment_exists(
        &self,
        request: Request<AssignmentIdRequest>,
    ) -> Result<Response<AssignmentIdResponse>, Status> {
        let uuid = Uuid::parse_str(&request.into_inner().assignment_id).unwrap();
        let id = AssignmentId(uuid);
        let ret = self.assignments.get(&id).map(|x| x.clone()).is_some();

        Ok(Response::new(AssignmentIdResponse { found: ret }))
    }

    async fn get_assignment(
        &self,
        request: Request<AssignmentIdRequest>,
    ) -> Result<Response<deep_project::Assignment>, Status> {
        let uuid = Uuid::parse_str(&request.into_inner().assignment_id).unwrap();
        let id = AssignmentId(uuid);
        if let Some(assignment) = &self.assignments.get(&id) {
            let ret = deep_project::Assignment {
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

impl From<deep_project::Assignment> for config::Assignment {
    fn from(assignment: deep_project::Assignment) -> Self {
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

impl From<HashMap<config::AssignmentId, config::Assignment>> for VecAssignmentsShort {
    fn from(thing: HashMap<config::AssignmentId, config::Assignment>) -> Self {
        let a = thing
            .into_iter()
            .map(|(id, a)| deep_project::AssignmentShort {
                name: a.name,
                assignment_id: id.0.to_string(),
            })
            .collect::<_>();
        VecAssignmentsShort { assignments: a }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    let addr = "[::1]:50051".parse().unwrap();
    let opt = Opt::from_args();
    let test = Tester::new(parse_config(&opt.config)?);
    println!("Tester listening on {}", addr);

    Server::builder()
        .add_service(TestServer::new(test))
        .serve(addr)
        .await?;

    Ok(())
}
