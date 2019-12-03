use std::io::Write;

use actix_web::{web, HttpResponse, Responder, ResponseError};
use log::info;
use tempfile::NamedTempFile;

use crate::api::{AssignmentResult, AssignmentShort, IliasId, Submission};
use crate::config::AssignmentId;
use crate::crash_test::{run, ScriptResult};
use crate::state::State;
use actix_web::error::JsonPayloadError;

fn inner_get_result(state: web::Data<State>, para: IliasId) -> Result<HttpResponse, Error> {
    let id = para;
    let mut rets = state.pending_results.write().unwrap();
    if let Some(ret) = rets.remove(&id) {
        return Ok(HttpResponse::Ok().json(ret));
    }
    Err(Error::NotFoundIliasId(id))
}

pub fn get_result(state: web::Data<State>, para: web::Path<String>) -> Result<HttpResponse, Error> {
    let str = para.into_inner();
    match str.parse::<u64>() {
        Ok(id) => inner_get_result(state, id.into()),
        Err(e) => Err(Error::Parameter(e.to_string(), "An integer64 is required")),
    }
}
pub fn add_submission(
    state: web::Data<State>,
    para: web::Json<Submission>,
) -> Result<HttpResponse, Error> {
    let para = para.into_inner();
    info!("Submisson: the parameter is: {:#?}", para);
    let mut script_file = NamedTempFile::new()?;
    script_file.write(&para.source_code.0.as_bytes())?;
    let mut rets = state.pending_results.write().unwrap();
    let config = state.config.clone();
    if rets.contains_key(&para.ilias_id) {
        return Err(Error::DuplicateIliasId);
    }
    //TODO better err handling
    if let Some(assignment) = config.get(&para.assigment_id) {
        let test_result = match run(assignment, script_file.into_temp_path().to_path_buf()) {
            ScriptResult::Correct => (true, None),
            ScriptResult::InCorrect(msg) => (false, Some(msg)),
        };
        rets.insert(
            para.ilias_id,
            AssignmentResult::new(test_result.0, test_result.1, None),
        );
    };
    Ok(HttpResponse::Created().body(""))
}

pub fn get_assignments(state: web::Data<State>) -> HttpResponse {
    let list = state
        .config
        .iter()
        .map(|(k, v)| v.into_short(*k))
        .collect::<Vec<AssignmentShort<'_>>>();
    HttpResponse::Ok().json(list)
}

pub fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello FH Dortmund")
}

pub fn version() -> web::Json<MetaJson> {
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
    fn error_response(&self) -> HttpResponse {
        let err = ErrJson::from(self);
        match self {
            Error::DuplicateIliasId => HttpResponse::Conflict().json(err),
            Error::NotFoundIliasId(_) => HttpResponse::NotFound().json(err),
            Error::Parameter(_, _) => HttpResponse::BadRequest().json(err),
            Error::Body(err) => HttpResponse::BadRequest().json(ErrSubmission {
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
    fn render_response(&self) -> HttpResponse {
        self.error_response()
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
