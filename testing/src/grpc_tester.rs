use crate::crash_test::{CrashTester, Error, Files, Stdout};
use crate::docker_api::DockerWrap;
use crate::{crash_test, fs_util, sema_wrap};
use futures::future;
use grpc_api::test_server::Test;
use grpc_api::{Assignment, AssignmentMsg, AssignmentResult, Script};
use log::info;
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
pub struct Tester {
    docker: sema_wrap::SemWrap<DockerWrap>,
}

impl Tester {
    pub fn new(docker: DockerWrap, max_sema: usize) -> Self {
        Tester {
            docker: sema_wrap::SemWrap::new(docker, max_sema),
        }
    }
}
#[tonic::async_trait]
impl Test for Tester {
    async fn run_test(
        &self,
        request: Request<AssignmentMsg>,
    ) -> Result<Response<AssignmentResult>, Status> {
        let req = request.into_inner();
        // TODO more and better err handling + log
        if let Some(assignment) = req.assignment {
            let reply = match self.inner_run_test(&assignment, &req.code_to_test).await {
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

impl Tester {
    async fn inner_run_test(
        &self,
        assignment: &Assignment,
        code_to_test: &str,
    ) -> Result<(), Error> {
        // TODO Fix always into thank you tonic
        let script_type: &Script = &assignment.script_type.into();
        info!(
            "running task: {}, type: {:#?}",
            &assignment.name, &script_type
        );
        let context_dir = fs_util::extract_files_include(&assignment.include_files).await?;
        let script_test_path =
            fs_util::new_tmp_script_file(assignment.script_type.into(), code_to_test)
                .map_err(Error::CantCreatTempFile)?
                .into_temp_path();
        let script_solution_path =
            fs_util::new_tmp_script_file(assignment.script_type.into(), &assignment.solution)
                .map_err(Error::CantCreatTempFile)?
                .into_temp_path();
        let docker_api = self.docker.acquire().await;
        let test_output = docker_api
            .test_in_container(
                &assignment.script_type.into(),
                &script_test_path,
                &context_dir.path(),
                &assignment.args,
            )
            .await?;
        let solution_context_dir =
            fs_util::extract_files_include(&assignment.include_files).await?;
        let solution_output = docker_api
            .test_in_container(
                &assignment.script_type.into(),
                &script_solution_path,
                &solution_context_dir.path(),
                &assignment.args,
            )
            .await?;
        // TODO without Vec<Box<dyn CrashTester>>, try!..
        let mut tests: Vec<Box<dyn CrashTester>> = Vec::new();

        tests.push(Stdout::boxed(solution_output, test_output));
        tests.push(Files::boxed(
            solution_context_dir.path().to_path_buf(),
            context_dir.path().to_path_buf(),
        ));
        let _ =
            future::try_join_all(tests.iter().map(|item| async move { item.test().await })).await?;
        Ok(())
    }
}
