//! The RPC testing server.
use crate::checker::{
    Checker, CustomScriptChecker, FilesChecker, RegexChecker, SortedChecker, StdoutChecker,
};
use crate::docker_api::DockerWrap;
use crate::error::{Error, IOError, SystemError};
use crate::{fs_util, sema_wrap};
use futures::future;
use grpc_api::test_server::Test;
use grpc_api::{Assignment, AssignmentMsg, AssignmentResult, RegexMode, Script, SortStdoutBy};
use log::info;
use tonic::{Request, Response, Status};
/// State
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
                Err(Error::InvalidTest(e)) => {
                    log::error!("Invalid test error_msg: {}", e);
                    AssignmentResult {
                        passed: false,
                        message: Some(e.to_string()),
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
                .map_err(|e| IOError::CreateFile(e))?
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
        test_output.status_success()?;
        log::info!("Test Output: {}", test_output);
        // only run solution if one of the 3 options is on
        let mut tests: Vec<Box<dyn Checker>> = Vec::new();
        if assignment.compare_fs_solution
            | assignment.compare_stdout_solution
            | assignment.compare_stdout_solution
            | assignment.custom_script.is_some()
        {
            let script_solution_path =
                fs_util::new_tmp_script_file(assignment.script_type.into(), &assignment.solution)
                    .map_err(|e| IOError::CreateFile(e))?
                    .into_temp_path();
            let solution_context_dir =
                fs_util::extract_files_include(&assignment.include_files).await?;

            let solution_output = docker_api
                .test_in_container(
                    &assignment.script_type.into(),
                    &script_solution_path,
                    &solution_context_dir.path(),
                    &assignment.args,
                )
                .await
                .map_err(|_e| SystemError::BadSampleSolution)?;
            log::info!("Solution Output: {}", solution_output);
            solution_output
                .status_success()
                .map_err(|_e| SystemError::BadSampleSolution)?;

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
        }

        let regex_mode = assignment.regex_mode.into();
        if regex_mode != RegexMode::UnknownRegex {
            tests.push(RegexChecker::boxed(
                assignment.regex.as_ref(),
                regex_mode,
                &test_output.stdout,
                &code_to_test.to_string(),
            ));
        }
        if let Some(custom_script) = &assignment.custom_script {
            tests.push(CustomScriptChecker::boxed(
                &custom_script,
                code_to_test,
                &test_output,
                &context_dir.path(),
            ))
        }

        let _ = future::try_join_all(tests.iter().map(|item| async move { item.check().await }))
            .await?;
        info!("testing done for assignment: {}", &assignment.name);
        Ok(())
    }
}
