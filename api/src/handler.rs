use crate::api::{AssignmentId, IliasId, Submission};
use crate::state::State;
use actix_web::error::JsonPayloadError;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use std::fmt::Debug;

use crate::deep_project::test_client::TestClient;
use crate::deep_project::{AssignmentIdRequest, AssignmentIdResponse, AssignmentMsg};

fn inner_get_result(state: web::Data<State>, para: IliasId) -> Result<HttpResponse, Error> {
    let id = para;
    if let Some(ret) = state.pending_results.remove(&id) {
        return Ok(HttpResponse::Ok().json(ret.1));
    }
    Err(Error::NotFoundIliasId(id))
}

pub async fn get_result(
    state: web::Data<State>,
    para: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let str = para.into_inner();
    match str.parse::<u64>() {
        Ok(id) => inner_get_result(state, id.into()),
        Err(e) => Err(Error::Parameter(e.to_string(), "An integer64 is required")),
    }
}
pub async fn add_submission(
    state: web::Data<State>,
    para: web::Json<Submission>,
) -> Result<HttpResponse, Error> {
    let para = para.into_inner();
    //let config = state.config.clone();
    dbg!(&para);
    let mut client = TestClient::connect("http://[::1]:50051").await.unwrap();

    if state.pending_results.contains_key(&para.ilias_id) {
        return Err(Error::DuplicateIliasId);
    }

    let a_req = tonic::Request::new(AssignmentIdRequest {
        assignment_id: para.assignment_id.0,
    });
    let assignment = client
        .get_assignment(a_req)
        .await
        .map_err(|_| Error::NotAssignment(para.assignment_id))?
        .into_inner();
    dbg!(&assignment);
    tokio::task::spawn(async move {
        let request = tonic::Request::new(AssignmentMsg {
            assignment: Some(assignment),
            source_code: para.source_code.0,
        });
        // TODO fix unwrap
        if let Ok() = client.run_test(request).await {
        } else {
        }
        match client.run_test(request).await {
            Ok(response) => state
                .pending_results
                .insert(para.ilias_id, response.into_inner()),
            Err(e) => log::info!("error from rpc {:?}", e),
        }
    });
    Ok(HttpResponse::Created().body(""))
}

// async fn wait_print_err<E: Debug>(e: E) {
//     tokio::time::delay_for(std::time::Duration::from_secs(3)).await;
//     log::info!("System Error. Waiting for 3 secs. {:?}", e);
// }

pub async fn get_assignments(_state: web::Data<State>) -> HttpResponse {
    let mut client = TestClient::connect("http://[::1]:50051").await.unwrap();
    let request = tonic::Request::new(());

    let response = client.get_assignments(request).await.unwrap();
    HttpResponse::Ok().json(response.into_inner().assignments)
}

pub async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Hello FH Dortmund")
}

pub async fn version() -> web::Json<MetaJson> {
    web::Json(MetaJson::new(0.2, EndPointStatus::Online))
}
#[derive(serde::Serialize)]
struct ErrJson {
    msg: String,
}
#[derive(serde::Serialize)]
struct ErrSubmission {
    msg: String,
    example: SubmissionExample,
}

impl From<&Error> for ErrJson {
    fn from(e: &Error) -> Self {
        ErrJson { msg: e.to_string() }
    }
}

#[derive(failure::Fail, Debug)]
pub enum Error {
    #[fail(display = "Generic Error {}.", _0)]
    General(Box<dyn std::error::Error + Sync + Send>),
    #[fail(display = "Duplicate Ilias ID.")]
    DuplicateIliasId,
    // maybe return the ilias id back
    #[fail(display = "No Results not found for given Ilias ID: {}.", _0)]
    NotFoundIliasId(IliasId),
    #[fail(display = "No Results not found for given assignment ID: {}.", _0)]
    NotAssignment(AssignmentId),
    #[fail(display = "Parameter error {}. {}.", _0, _1)]
    Parameter(String, &'static str),
    #[fail(display = "Request body error. {:?}.", _0)]
    Body(JsonPayloadError),
}
impl<T> From<T> for Error
where
    T: std::error::Error + Sync + Send + 'static,
{
    fn from(error: T) -> Self {
        Error::General(Box::new(error))
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::DuplicateIliasId => StatusCode::CONFLICT,
            Error::NotFoundIliasId(_) | Error::NotAssignment(_) => StatusCode::NOT_FOUND,
            Error::Parameter(_, _) => StatusCode::BAD_REQUEST,
            Error::Body(_err) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let err = ErrJson::from(self);
        let code = self.status_code();
        let mut response = HttpResponse::build(code);
        match self {
            Error::DuplicateIliasId
            | Error::NotFoundIliasId(_)
            | Error::Parameter(_, _)
            | Error::NotAssignment(_) => response.json(err),
            Error::Body(err) => response.json(ErrSubmission {
                msg: err.to_string(),
                example: SubmissionExample::new(
                    2009.into(),
                    "ZWNobyAiSGFsbG8iID4+IGhhbGxvLnR4dAo=",
                    AssignmentId(64),
                ),
            }),
            _ => HttpResponse::InternalServerError().json(err),
        }
    }
}

#[derive(Debug, serde::Serialize, derive_more::Constructor)]
#[serde(rename_all = "camelCase")]

pub struct SubmissionExample {
    pub ilias_id: IliasId,
    pub source_code: &'static str,
    pub assignment_id: AssignmentId,
}

#[derive(serde::Serialize, Debug, Clone, derive_more::Constructor)]
#[serde(rename_all = "camelCase")]
pub struct MetaJson {
    pub version: f32,
    pub status: EndPointStatus,
}

#[derive(serde::Serialize, Debug, Clone)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
pub enum EndPointStatus {
    Online,
    Maintenance,
    Offline,
}

impl Default for EndPointStatus {
    fn default() -> Self {
        EndPointStatus::Online
    }
}
