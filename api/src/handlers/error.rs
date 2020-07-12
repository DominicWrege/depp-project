//! Error handling using [failure](https://docs.rs/crate/failure) as error library.
use crate::api::{IliasId, SubmissionExample};
use crate::rpc_conf::RpcMeta;
use actix_web::error::JsonPayloadError;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use grpc_api::AssignmentId;
use uuid::Uuid;

#[derive(serde::Serialize)]
pub struct ErrJson {
    msg: String,
}
#[derive(serde::Serialize)]
pub struct ErrSubmission {
    msg: String,
    example: SubmissionExample,
}

impl From<&Error> for ErrJson {
    fn from(e: &Error) -> Self {
        ErrJson { msg: e.to_string() }
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::DuplicateIliasId => StatusCode::CONFLICT,
            Error::NotFoundIliasId(_) | Error::NotAssignment(_) => StatusCode::NOT_FOUND,
            Error::BadRequest => StatusCode::BAD_REQUEST,
            Error::Submission(_e) => StatusCode::BAD_REQUEST,
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
            Error::Submission(_e) => response.json(ErrSubmission {
                msg: self.to_string(),
                example: SubmissionExample::new(
                    IliasId::default(),
                    "IyEgL2Jpbi9iYXNoCmlucHV0PSQxCmxlbmd0aD0keyNpbnB1dH0KcmV2ZXJzZT0iIgpmb3IgaSBpbiAkKHNlcSAxICRsZW5ndGggKQpkbwoJcmV2ZXJzZSs9JHtpbnB1dDokbGVuZ3RoLWk6MX0KZG9uZQplY2hvICJBdXNnYWJlIHVtZ2VrZWhydDogJHJldmVyc2Ui",
                    Uuid::parse_str("936DA01F9ABD4d9d80C702AF85C822A8").unwrap(),
                ),
            }),
            _ => HttpResponse::InternalServerError().json(err),
        }
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
    #[fail(
        display = "Incorrect json received error: {}. Maybe there are some fields missing or the types does not match.",
        _0
    )]
    RpcOffline { reason: RpcMeta },
    #[fail(display = "Bad request")]
    BadRequest,
    #[fail(display = " Wrong credentials")]
    Unauthorized,
    #[fail(display = "{}", _0)]
    Submission(BadSubmission),
}
#[derive(failure::Fail, Debug)]
pub enum BadSubmission {
    #[fail(display = "json err: {:?}", _0)]
    Json(String),
    #[fail(display = "Wrong content type expected application/json.")]
    ContentType,
    #[fail(display = "{:?}", _0)]
    Other(String),
}

impl From<BadSubmission> for Error {
    fn from(bad_err: BadSubmission) -> Self {
        Error::Submission(bad_err)
    }
}

impl<T> From<T> for Error
where
    T: std::error::Error + Sync + Send + 'static,
{
    fn from(error: T) -> Self {
        Error::General(Box::new(error))
    }
}

pub fn sub_extractor(ae: actix_web::error::Error) -> Error {
    match ae.as_error::<JsonPayloadError>() {
        Some(inner_e) => match inner_e {
            JsonPayloadError::ContentType => BadSubmission::ContentType.into(),
            JsonPayloadError::Deserialize(e) => BadSubmission::Json(e.to_string()).into(),
            _ => BadSubmission::Other(inner_e.to_string()).into(),
        },
        None => Error::Submission(BadSubmission::Other(ae.to_string())),
    }
}

impl Error {
    pub(crate) fn into_actix_web_err(self) -> actix_web::Error {
        actix_web::error::ErrorUnauthorized(self.to_string())
    }
}
