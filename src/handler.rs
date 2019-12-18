use crate::api::{AssignmentId, AssignmentResult, AssignmentShort, IliasId, Submission};
use crate::crash_test;
use crate::crash_test::run;
use crate::state::State;
use actix_web::error::JsonPayloadError;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};

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
    let config = state.config.clone();
    if state.pending_results.contains_key(&para.ilias_id) {
        return Err(Error::DuplicateIliasId);
    }
    if let Some(assignment) = config.get(&para.assigment_id).map(|x| x.clone()) {
        tokio::task::spawn(async move {
            let mut attempts_counter = 0;
            loop {
                if attempts_counter >= 10 {
                    panic!("Server Stopped! System might be corrupted.")
                }
                match run(&assignment, &para.source_code).await {
                    Err(crash_test::Error::ReadFile(e, _))
                    | Err(crash_test::Error::CantCreatTempFile(e)) => {
                        tokio::time::delay_for(std::time::Duration::from_secs(3)).await;
                        log::info!("System Error. Waiting for 3 secs. {}", e);
                    }
                    Err(e) => {
                        break state.pending_results.insert(
                            para.ilias_id,
                            AssignmentResult::new(false, Some(e.to_string()), None),
                        );
                    }
                    Ok(_) => {
                        break state
                            .pending_results
                            .insert(para.ilias_id, AssignmentResult::new(true, None, None));
                    }
                }
                attempts_counter = attempts_counter + 1;
            }
        });
    };

    Ok(HttpResponse::Created().body(""))
}

pub async fn get_assignments(state: web::Data<State>) -> HttpResponse {
    let list = state
        .config
        .iter()
        .map(|(k, v)| v.into_short(*k))
        .collect::<Vec<AssignmentShort<'_>>>();
    HttpResponse::Ok().json(list)
}

pub async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Hello FH Dortmund")
}

pub async fn version() -> web::Json<MetaJson> {
    web::Json(MetaJson::new(0.1, EndPointStatus::Online))
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
            Error::NotFoundIliasId(_) => StatusCode::NOT_FOUND,
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
            Error::DuplicateIliasId | Error::NotFoundIliasId(_) | Error::Parameter(_, _) => {
                response.json(err)
            }
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
    pub assigment_id: AssignmentId,
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
