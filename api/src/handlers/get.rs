//! This is an REST API using only ```JSON```.

use crate::api::{AssignmentShort, Status, Version};
use crate::handlers::error::Error;
use crate::state::State;
use actix_web::http::{Method, StatusCode};
use actix_web::{web, HttpRequest, HttpResponse};
use tokio_postgres::row::Row;
/// Return test result for a script or
/// 404 if the given IliasId does not exists.
pub async fn get_result(
    req: HttpRequest,
    state: web::Data<State>,
    para: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let id = para.into_inner().into();
    match req.method().to_owned() {
        Method::POST => {
            if state.pending_results.remove(&id).is_some() {
                Ok(HttpResponse::Ok().body(""))
            } else {
                Err(Error::NotFoundIliasId(id))
            }
        }
        Method::GET => {
            let state = state.into_inner();
            if state.to_test_assignments.read().await.contains(&id) {
                return Ok(HttpResponse::new(StatusCode::ACCEPTED));
            }
            if let Some(ret) = state
                .pending_results
                .get(&id)
                .map(|ret| ret.value().clone())
            {
                Ok(HttpResponse::Ok().json(ret))
            } else {
                Err(Error::NotFoundIliasId(id))
            }
        }
        _ => Err(Error::BadRequest),
    }
}
/// Return all assignments which currently active.
pub async fn get_assignments(state: web::Data<State>) -> Result<HttpResponse, Error> {
    let client = &state.db_pool.get().await?;
    let query = r#"SELECT format('%s/%s (%s)', exercise.description, assignment_name, script_type) as name, uuid
                        FROM assignment JOIN exercise
                        ON assignment.exercise_id = exercise.id
                        WHERE assignment.active is true
                        ORDER BY name;"#;
    let rows = client.query(query, &[]).await?;
    Ok(HttpResponse::Ok().json(
        rows.into_iter()
            .map(|r| AssignmentShort::from(r))
            .collect::<Vec<_>>(),
    ))
}

impl From<Row> for AssignmentShort {
    fn from(r: Row) -> Self {
        Self {
            id: r.get("uuid"),
            name: r.get("name"),
        }
    }
}
#[doc(hidden)]
pub async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Hello FH Dortmund")
}
/// Return API version.
pub async fn version() -> HttpResponse {
    HttpResponse::Ok().json(Version {
        version: env!("CARGO_PKG_VERSION"),
    })
}
/// Return status of booth RPC Endpoints.
pub async fn status(state: web::Data<State>) -> HttpResponse {
    let rpc = &state.rpc_conf;
    let status = rpc.status().await;
    HttpResponse::Ok().json(Status {
        linux_rpc_status: status.linux,
        windows_rpc_status: status.windows,
    })
}
