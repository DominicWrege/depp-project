use actix_web::dev::ServiceRequest;
use actix_web::error::JsonPayloadError;
use actix_web::http::{Method, StatusCode};
use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use actix_web_httpauth::extractors::basic::BasicAuth;
use std::fmt::Debug;
use uuid::Uuid;

use crate::api::{IliasId, Submission};
use crate::state::{get_rpc_status, Meta, State};

use grpc_api::test_client::TestClient;
use grpc_api::{AssignmentId, AssignmentMsg};
use sha2::Digest;

use crate::assignment;

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
pub async fn add_submission(
    state: web::Data<State>,
    para: web::Json<Submission>,
) -> Result<HttpResponse, Error> {
    let para = para.into_inner();
    //let config = state.config.clone();

    let assignment = assignment::get_assignment(&state.db_pool, &para.assignment_id)
        .await
        .map_err(|_| Error::NotAssignment(para.assignment_id))?;
    //dbg!(&assignment);
    let mut client = TestClient::connect(state.rpc_url.as_str().to_owned())
        .await
        .map_err(|_| Error::RpcOffline)?;

    // TODO only one test client

    if state.pending_results.contains_key(&para.ilias_id)
        || state
            .to_test_assignments
            .read()
            .await
            .contains(&para.ilias_id)
    {
        return Err(Error::DuplicateIliasId);
    }

    tokio::task::spawn(async move {
        let state = state.into_inner();
        let ilias_id = para.ilias_id;
        state
            .to_test_assignments
            .write()
            .await
            .insert(ilias_id.clone());
        let request = tonic::Request::new(AssignmentMsg {
            assignment: Some(assignment),
            code_to_test: para.source_code.0,
        });
        match client.run_test(request).await {
            Ok(response) => {
                state
                    .pending_results
                    .insert(ilias_id.clone(), response.into_inner());
                state.to_test_assignments.write().await.remove(&ilias_id);
            }
            Err(e) => {
                log::error!("error from rpc {:?}", e);
            }
        }
    });
    Ok(HttpResponse::Created().body(""))
}

pub async fn get_assignments(state: web::Data<State>) -> Result<HttpResponse, Error> {
    let pool = &state.db_pool;
    let assignments = assignment::get_assignments(&pool).await?;
    Ok(HttpResponse::Ok().json(assignments))
}

pub async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Hello FH Dortmund")
}

pub async fn version(state: web::Data<State>) -> HttpResponse {
    HttpResponse::Ok().json(Meta::new(
        "0.3",
        &get_rpc_status(&state.inner.rpc_url).await,
    ))
}

pub async fn auth(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, actix_web::Error> {
    let state: web::Data<State> = req.app_data().unwrap();
    match credentials.password() {
        Some(cred) => {
            let pwd = sha2::Sha256::digest(cred.as_bytes()).to_vec();
            if credentials.user_id() == state.credentials.username()
                && pwd == state.credentials.password()
            {
                Ok(req)
            } else {
                Err(Error::Unauthorized.into_actix_web_err())
            }
        }
        None => Err(Error::Unauthorized.into_actix_web_err()),
    }
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
    #[fail(display = "Generic Error {}", _0)]
    General(Box<dyn std::error::Error + Sync + Send>),
    #[fail(display = "Duplicate IliasID")]
    DuplicateIliasId,
    // maybe return the ilias id back
    #[fail(display = "No Results not found for given IliasID: {}", _0)]
    NotFoundIliasId(IliasId),
    #[fail(display = "No Results not found for given AssignmentID: {}", _0)]
    NotAssignment(AssignmentId),
    #[fail(display = "Request body error. {:?}", _0)]
    Body(JsonPayloadError),
    #[fail(display = "The testing server is offline")]
    RpcOffline,
    #[fail(display = "Bad request")]
    BadRequest,
    #[fail(display = " Wrong credentials")]
    Unauthorized,
}
impl<T> From<T> for Error
where
    T: std::error::Error + Sync + Send + 'static,
{
    fn from(error: T) -> Self {
        Error::General(Box::new(error))
    }
}

impl Error {
    fn into_actix_web_err(self) -> actix_web::Error {
        actix_web::error::ErrorUnauthorized(self.to_string())
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::DuplicateIliasId => StatusCode::CONFLICT,
            Error::NotFoundIliasId(_) | Error::NotAssignment(_) => StatusCode::NOT_FOUND,
            Error::BadRequest => StatusCode::BAD_REQUEST,
            Error::Body(_err) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let err = ErrJson::from(self);
        let code = self.status_code();
        let mut response = HttpResponse::build(code);
        match self {
            Error::DuplicateIliasId | Error::NotFoundIliasId(_) | Error::NotAssignment(_) => {
                response.json(err)
            }
            Error::Body(err) => response.json(ErrSubmission {
                msg: err.to_string(),
                example: SubmissionExample::new(
                    IliasId::default(),
                    "ZWNobyAiSGFsbG8iID4+IGhhbGxvLnR4dAo=",
                    Uuid::parse_str("936DA01F9ABD4d9d80C702AF85C822A8").unwrap(),
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
    pub assignment_id: Uuid,
}
