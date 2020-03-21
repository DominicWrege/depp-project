use crate::assignments::file;
use crate::db::{rows_into, ScriptType};
use crate::error::HttpError;
use crate::handler::{render_template, HttpResult};
use crate::template::TEMPLATES;
use crate::{db, State};
use actix_web::web;
use serde::{Deserialize, Deserializer};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(serde::Deserialize, serde::Serialize, Debug, PostgresMapper)]
#[pg_mapper(table = "assignment_exercise")]
pub struct AssignmentExercise {
    uuid: uuid::Uuid,
    name: String,
    script_type: ScriptType,
    description: String,
    exercise_name: String,
}

pub fn parse_path(path: &str) -> Result<i32, HttpError> {
    path.parse::<i32>()
        .map_err(|_| HttpError::WrongParameter(String::from(path)))
}

pub async fn all_assignments_for_exercise(
    path: web::Path<String>,
    data: web::Data<State>,
) -> HttpResult {
    let id = parse_path(&path.into_inner())?;
    let client = &data.db_pool.get().await?;
    let stmt = client
        .prepare(
            r#"
            SELECT a.assignment_name as name, script_type, e.description as exercise_name, a.description, a.uuid
            FROM assignment a INNER JOIN exercise e ON a.exercise_id = e.id
            WHERE e.id = $1"#,
        )
        .await?;
    let rows = client.query(&stmt, &[&id]).await?;
    // TODO into.,...
    let assignments: Vec<AssignmentExercise> = rows_into(rows);
    let mut context = tera::Context::new();
    context.insert("assignments", &assignments);
    render_template(&TEMPLATES, "assignments_list.html", &context)
}

fn to_status<'de, D>(deserial: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserial) {
        Ok(s) if s == "on" => Ok(true),
        _ => Ok(false),
    }
}

fn to_args<'de, D>(deserial: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserial) {
        Ok(s) => {
            let s = s
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>()
                .split(",")
                .map(|s| s.to_string())
                .collect::<Vec<_>>();
            Ok(s)
        }
        _ => Ok(vec![]),
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PostgresMapper)]
#[pg_mapper(table = "assignment")]
pub struct Assignment {
    #[serde(skip_deserializing)]
    pub uuid: uuid::Uuid,
    pub name: String,
    pub solution: String,
    pub script_type: ScriptType,
    pub description: String,
    #[serde(deserialize_with = "to_status", default)]
    pub active: bool,
    pub exercise_id: i32,
    #[serde(deserialize_with = "to_args")]
    pub args: Vec<String>,
    #[serde(skip_deserializing)]
    pub include_files: Vec<u8>,
}

pub async fn single_assignment(path: web::Path<uuid::Uuid>, data: web::Data<State>) -> HttpResult {
    let uuid = path.into_inner();
    let pool = &data.db_pool;
    let client = pool.get().await?;
    let stmt = client.prepare(r#"SELECT assignment_name as name, script_type, active, include_files, solution, description, uuid, args, exercise_id
                                                              FROM assignment
                                                              WHERE uuid = $1"#).await?;
    let row = client.query_one(&stmt, &[&uuid]).await?;
    let assignment =
        Assignment::from_row(row).map_err(|_e| HttpError::NotFound("Assignment".into()))?;
    let files = file::ls_zip_content(&assignment.include_files)?;
    let scripts = db::get_script_types(&pool).await?;
    let exercises = db::get_all_exercises(&pool).await?;
    let mut context = tera::Context::new();
    context.insert("files", &files);
    context.insert("assignment", &assignment);
    context.insert("scripts", &scripts);
    context.insert("exercises", &exercises);
    render_template(&TEMPLATES, "assignment_view.html", &context)
}
