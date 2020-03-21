use crate::template::TEMPLATES;
use actix_web::http::StatusCode;
use actix_web::{error, HttpResponse};
use std::borrow::Borrow;

#[derive(Debug, failure::Fail)]
pub enum HttpError {
    #[fail(display = "Internal Server error {}", _0)]
    General(Box<dyn std::error::Error + Sync + Send>),
    /*    #[fail(display = "DB error {}", _0)]
    Db(DbError),*/
    #[fail(display = "Found: {:#?} expected: i32", _0)]
    WrongParameter(String),
    #[fail(display = "Wrong Password!")]
    WrongPassword,
    #[fail(display = "{} was not found.", _0)]
    NotFound(String),
}

impl<T> From<T> for HttpError
where
    T: std::error::Error + Sync + Send + 'static,
{
    fn from(error: T) -> Self {
        HttpError::General(Box::new(error))
    }
}

impl error::ResponseError for HttpError {
    fn status_code(&self) -> StatusCode {
        match self {
            HttpError::NotFound(_) => StatusCode::NOT_FOUND,
            HttpError::WrongParameter(_) => StatusCode::BAD_REQUEST,
            HttpError::WrongPassword => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        log::error!("{:}", &self);
        let mut c = tera::Context::new();
        c.insert("hide_top_btns", &false);
        let tera = TEMPLATES.borrow();
        let body = match self {
            HttpError::NotFound(_) | HttpError::WrongParameter(_) => {
                c.insert("error_title", "Bad Request");
                c.insert("error_msg", &self.to_string());
                tera.render("error.html", &c).unwrap()
            }
            HttpError::WrongPassword => {
                c.insert("error_msg", &self.to_string());
                tera.render("login.html", &c).unwrap()
            }
            HttpError::General(_e) => {
                c.insert("error_title", "Internal Server Error");
                c.insert("error_msg", &self.to_string());
                tera.render("error.html", &c).unwrap()
            }
        };

        HttpResponse::build(self.status_code())
            .content_type("text/html; charset=utf-8")
            .body(body)
    }
}
