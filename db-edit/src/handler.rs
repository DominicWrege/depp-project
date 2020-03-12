use crate::db::{Exercise, ScriptType};
use crate::{db, State};
use actix_multipart::{Field, Multipart};
use actix_web::http::StatusCode;
use actix_web::{error, http, web, HttpResponse};
use db_lib::DbError;
use futures::StreamExt;
use mime;
use std::collections::HashMap;
use std::str::FromStr;
use tera::Tera;
type HttpResult = Result<HttpResponse, HttpError>;

#[derive(serde::Deserialize, Debug)]
pub struct AssignmentForm<'a> {
    pub name: &'a str,
    pub solution: String,
    pub script_type: ScriptType,
    #[serde(borrow)]
    pub args: Vec<&'a str>,
    pub exercise_id: i32,
    pub include_files: &'a [u8],
}

pub async fn index(data: web::Data<State>) -> HttpResult {
    let pool = &data.db_pool;
    let exercises = db::get_exercises_with_assignments(&pool).await?;
    let mut context = tera::Context::new();
    context.insert("exercises", &exercises);
    render_template(&data.temp, "index.html", &context)
}

pub async fn new_assignment(data: web::Data<State>, mut payload: Multipart) -> HttpResult {
    let mut text_fields: HashMap<_, _> = HashMap::new();
    let mut zip_file: Vec<u8> = vec![];
    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();

        if field.content_type().subtype() == mime::OCTET_STREAM {
            if let Some((k, v)) = convert_field(&mut field).await {
                text_fields.insert(k, v);
            }
        } else if field.content_type().subtype() == "zip" {
            if let Some(Ok(s)) = field.next().await {
                zip_file = s.to_vec();
            }
        }
    }

    let assign = into_assignment_form(&mut text_fields, &zip_file);
    db::insert_assignment(&data.db_pool, &assign).await?;
    Ok(HttpResponse::Found()
        .header(
            http::header::LOCATION,
            format!("exercise/{}", &assign.exercise_id),
        )
        .finish())
}

fn fix_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}

fn into_assignment_form<'a>(
    h: &'a mut HashMap<String, String>,
    zip: &'a [u8],
) -> AssignmentForm<'a> {
    AssignmentForm {
        name: h.get("name").unwrap(),
        solution: fix_newlines(h.get("solution").unwrap()),
        script_type: ScriptType::from_str(h.get("script_type").unwrap()).unwrap(),
        args: if let Some(a) = h.get("args") {
            a.split(',').collect::<Vec<_>>()
        } else {
            vec![]
        },
        exercise_id: h.get("exercise_id").unwrap().parse::<i32>().unwrap(),
        include_files: zip,
    }
}

async fn convert_field(field: &mut Field) -> Option<(String, String)> {
    let content_disposition = &field.content_disposition().unwrap();
    if let Some(f_name) = content_disposition.get_name() {
        if let Some(s) = field.next().await {
            if let Ok(v) = String::from_utf8(s.unwrap_or_default().to_vec()) {
                return Some((f_name.to_string(), v));
            } else {
                return None;
            }
        }
    }
    None
}

fn read_zip(buf: &[u8]) {
    //let mut reader = tokio::io::BufReader::new(buf);
    let reader = std::io::Cursor::new(buf);
    if let Ok(mut zip) = zip::ZipArchive::new(reader) {
        for i in 0..zip.len() {
            let file = zip.by_index(i).unwrap();
            println!("Filename: {}", file.name());
            /*        let first_byte = file.bytes().next().unwrap();
            println!("{}", first_byte);*/
        }
    }
}

pub async fn get_assignments_for_exercise(
    path: web::Path<String>,
    data: web::Data<State>,
) -> HttpResult {
    let path = path.into_inner();
    let id = path
        .parse::<i32>()
        .map_err(|_| HttpError::CourseParameter(path))?;
    let assignments = db::get_assignments_for_exercise(&data.db_pool, id).await?;
    read_zip(assignments.get(0).unwrap().include_files.as_ref());
    let mut context = tera::Context::new();
    context.insert("assignments", &assignments);
    render_template(&data.temp, "assignments_views.html", &context)
}

pub async fn assignment_form(data: web::Data<State>) -> HttpResult {
    let pool = &data.db_pool;
    let scripts = db::get_script_types(&pool).await?;
    let exercises = db::get_all_exercises(&pool).await?;
    let mut context = tera::Context::new();
    context.insert("exercises", &exercises);
    context.insert("scripts", &scripts);
    render_template(&data.temp, "assignment_form.html", &context)
}

fn render_template(tera: &Tera, temp_name: &'static str, context: &tera::Context) -> HttpResult {
    let b = tera.render(temp_name, &context)?;
    Ok(HttpResponse::Ok().body(b))
}

#[derive(Debug, err_derive::Error, derive_more::From)]
pub enum HttpError {
    #[error(display = "db error {}", _0)]
    Db(DbError),
    #[error(display = "found: {:#?} expected: i32", _0)]
    CourseParameter(String),
    #[error(display = "error in template: {}", _0)]
    Template(tera::Error),
}

impl error::ResponseError for HttpError {
    fn status_code(&self) -> StatusCode {
        match self {
            HttpError::Db(_) | HttpError::Template(_) => StatusCode::INTERNAL_SERVER_ERROR,
            HttpError::CourseParameter(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        eprintln!("error {}", &self);
        let tera = parse_template("templates/*.html");
        let mut c = tera::Context::new();
        let body = match self {
            HttpError::Template(_) => "<h1>Template Error<h1>".into(),
            HttpError::Db(_) => tera.render("server_error.html", &c).unwrap(),
            HttpError::CourseParameter(_) => {
                c.insert("error_msg", &self.to_string());
                tera.render("bad_request_error.html", &c).unwrap()
            }
        };
        HttpResponse::build(self.status_code())
            .content_type("text/html; charset=utf-8")
            .body(body)
    }
}

pub fn parse_template(temp_name: &'static str) -> Tera {
    match Tera::new(temp_name) {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    }
}
