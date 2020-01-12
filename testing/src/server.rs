use std::collections::HashMap;
use std::path::Path;

mod config;
mod crash_test;
mod fs_util;
mod script;

use deep_project::test_server::{Test, TestServer};
use deep_project::{
    AssignmentIdRequest, AssignmentIdResponse, AssignmentMsg, AssignmentResult, AssignmentShort,
    Script, VecAssignmentsShort,
};
use structopt::StructOpt;
use tonic::{transport::Server, Request, Response, Status};

//use base64;
use config::{parse_config, Assignment, AssignmentId};

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
    assignments: HashMap<AssignmentId, Assignment>,
}

impl Tester {
    fn new(assignments: HashMap<AssignmentId, Assignment>) -> Self {
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
        let id = AssignmentId(msg.assignment_id);
        // TODO fix it
        if let Some(assignment) = &self.assignments.get(&id).map(|x| x.clone()) {
            // let _ = crash_test::run(&assignment, &msg.src_code).await.unwrap();

            let reply = match crash_test::run(&assignment, &msg.src_code).await {
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
            // TODO terrible
            Ok(Response::new(AssignmentResult {
                passed: true,
                message: None,
                mark: None,
            }))
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
        let id = AssignmentId(request.into_inner().assignment_id);
        let ret = self.assignments.get(&id).map(|x| x.clone()).is_some();

        Ok(Response::new(AssignmentIdResponse { found: ret }))
    }
}

impl From<HashMap<config::AssignmentId, config::Assignment>> for VecAssignmentsShort {
    fn from(thing: HashMap<config::AssignmentId, config::Assignment>) -> Self {
        let a = thing
            .into_iter()
            .map(|(id, a)| AssignmentShort {
                name: a.name,
                assignment_id: id.0,
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
