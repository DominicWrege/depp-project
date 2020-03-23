use crate::error::HttpError;

use actix_web::{http, HttpResponse};

pub type HttpResult = Result<HttpResponse, HttpError>;

use crate::PATH_PREFIX;
use tera::Tera;

pub fn render_template(
    tera: &Tera,
    temp_name: &'static str,
    context: &tera::Context,
) -> HttpResult {
    let b = tera.render(temp_name, &context)?;
    Ok(HttpResponse::Ok().body(b))
}

pub fn redirect<P>(path: P) -> HttpResponse
where
    P: AsRef<str>,
    P: std::fmt::Display,
{
    HttpResponse::Found()
        .header(http::header::LOCATION, format!("{}{}", PATH_PREFIX, path))
        .finish()
}

pub fn redirect_home() -> HttpResponse {
    redirect("")
}
