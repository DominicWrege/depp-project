use crate::api::Submission;
use crate::handlers::error::{sub_extractor, Error};
use crate::state::State;
use actix_web::{web, HttpResponse};
use deadpool_postgres::Pool;
use grpc_api::test_client::TestClient;
use grpc_api::{Assignment, AssignmentMsg, AssignmentResult};
/*
use tonic::transport::Channel;
*/
use uuid::Uuid;

pub async fn add_submission(
    state: web::Data<State>,
    json: Result<web::Json<Submission>, actix_web::error::Error>,
) -> Result<HttpResponse, Error> {
    let submission = json.map_err(sub_extractor)?;
    let assignment = db_assignment(&state.db_pool, &submission.assignment_id)
        .await
        .map_err(|_| Error::NotAssignment(submission.assignment_id))?;

    if state.pending_results.contains_key(&submission.ilias_id)
        || state
            .to_test_assignments
            .read()
            .await
            .contains(&submission.ilias_id)
    {
        return Err(Error::DuplicateIliasId);
    }
    let rpc = state.rpc_conf.meta(&assignment.script_type.into()).clone();

    let mut client = TestClient::connect(rpc.rpc_url.to_string())
        .await
        .map_err(|_| Error::RpcOffline {
            reason: rpc.clone(),
        })?;

    tokio::task::spawn(async move {
        let state = state.into_inner();
        let ilias_id = &submission.ilias_id;
        state
            .to_test_assignments
            .write()
            .await
            .insert(ilias_id.clone());
        let request = tonic::Request::new(AssignmentMsg {
            assignment: Some(assignment),
            code_to_test: submission.source_code.0.clone(),
        });

        match client.run_test(request).await {
            Ok(response) => {
                state
                    .pending_results
                    .insert(ilias_id.clone(), response.into_inner());
            }
            Err(e) => {
                state.pending_results.insert(
                    ilias_id.clone(),
                    AssignmentResult {
                        passed: false,
                        message: Some(e.to_string()),
                        valid: false,
                    },
                );
            }
        }
        state.to_test_assignments.write().await.remove(&ilias_id);
    });
    Ok(HttpResponse::Created().body(""))
}

async fn db_assignment(pool: &Pool, uuid: &Uuid) -> Result<Assignment, Error> {
    let client = pool.get().await?;
    let stmt = client
        .prepare(
            r#"SELECT assignment_name, script_type, include_files, solution, args, compare_fs_solution, 
                                compare_stdout_solution, regex, regex_check_mode, sort_stdout, custom_script
                    FROM assignment 
                    WHERE assignment.uuid = $1;"#,
        )
        .await?;
    let row = client
        .query_one(&stmt, &[uuid])
        .await
        .map_err(|_| Error::NotAssignment(*uuid))?;
    Ok(Assignment::from(&row))
}
