use crate::api::Submission;
use crate::handlers::error::Error;
use crate::state::State;
use actix_web::FromRequest;
use actix_web::{web, HttpRequest, HttpResponse};
use deadpool_postgres::Pool;
use grpc_api::test_client::TestClient;
use grpc_api::Assignment;
use grpc_api::AssignmentMsg;
/*
use tonic::transport::Channel;
*/
use uuid::Uuid;

pub async fn add_submission(
    state: web::Data<State>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let submission = web::Json::<Submission>::extract(&req).await.map_err(|e| {
        log::warn!("{}", e);
        Error::BadJson(e.to_string())
    })?;

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
                state.to_test_assignments.write().await.remove(&ilias_id);
            }
            Err(e) => {
                log::error!("from RPC {:#?}", e);
            }
        }
    });
    Ok(HttpResponse::Created().body(""))
}

async fn db_assignment(pool: &Pool, uuid: &Uuid) -> Result<Assignment, Error> {
    let client = pool.get().await?;
    let stmt = client
        .prepare(
            r#"SELECT assignment_name, script_type, include_files, solution, args
                    FROM assignment 
                    WHERE assignment.uuid = $1"#,
        )
        .await?;
    let row = client
        .query_one(&stmt, &[uuid])
        .await
        .map_err(|_| Error::NotAssignment(*uuid))?;
    Ok(Assignment::from(&row))
}
