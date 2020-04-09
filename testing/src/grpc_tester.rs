use crate::checker::{
    Checker, CustomScriptChecker, FilesChecker, RegexChecker, SortedChecker, StdoutChecker,
};
use crate::docker_api::DockerWrap;
use crate::error::Error;
use crate::{fs_util, sema_wrap};
use futures::future;
use grpc_api::test_server::Test;
use grpc_api::{Assignment, AssignmentMsg, AssignmentResult, RegexMode, Script, SortStdoutBy};
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
        if let Some(assignment) = req.assignment {
            let reply = match self.inner_run_test(&assignment, &req.code_to_test).await {
                Err(Error::CantCreatTempFile(e))
                | Err(Error::Copy(e))
                | Err(Error::FailedRunCustomScript(e)) => {
                    let msg = format!(
                        "Error while creating a tempfile or copying files or failed to run custom script. The server has to stop. error: {}", e
                    );
                    log::error!("{}", &msg);
                    AssignmentResult {
                        passed: false,
                        message: Some(msg),
                        valid: false,
                    }
                }
                Err(Error::Docker(e)) | Err(Error::InvalidRegex(e)) => {
                    let msg = format!(
                        "Some error with the docker API or the given regex is invalid. The server has to stop. error {}",
                        e
                    );
                    log::error!("{}", &msg);
                    AssignmentResult {
                        passed: false,
                        message: Some(msg),
                        valid: false,
                    }
                }
                Err(e) => AssignmentResult {
                    passed: false,
                    message: Some(e.to_string()),
                    valid: true,
                },
                Ok(_) => AssignmentResult {
                    passed: true,
                    message: None,
                    valid: true,
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
        // TODO Fix always into thank you grpc
        let script_type: &Script = &assignment.script_type.into();
        info!(
            "running test assignment name: {}, type: {:#?}",
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
        test_output.status_success()?;
        log::info!("now checking");
        let mut tests: Vec<Box<dyn Checker>> = Vec::new();
        let regex_mode = assignment.regex_mode.into();
        if regex_mode != RegexMode::UnknownRegex {
            tests.push(RegexChecker::boxed(
                assignment.regex.as_ref(),
                regex_mode,
                &test_output.stdout,
                &code_to_test.to_string(),
            ));
        }

        if assignment.compare_fs_solution {
            tests.push(FilesChecker::boxed(
                solution_context_dir.path().to_path_buf(),
                context_dir.path().to_path_buf(),
            ));
        }

        if assignment.compare_stdout_solution {
            tests.push(StdoutChecker::boxed(
                &solution_output.stdout,
                &test_output.stdout,
            ));
        }
        let sort_stdout_by = assignment.sort_stdout.into();
        if sort_stdout_by != SortStdoutBy::UnknownSort {
            tests.push(SortedChecker::boxed(&test_output.stdout, sort_stdout_by))
        }

        if let Some(custom_script) = &assignment.custom_script {
            tests.push(CustomScriptChecker::boxed(
                &custom_script,
                code_to_test,
                test_output,
                solution_output,
                &context_dir.path(),
            ))
        }

        let _ = future::try_join_all(tests.iter().map(|item| async move { item.check().await }))
            .await?;
        info!("testing done assignment name: {}", &assignment.name);
        Ok(())
    }
}
