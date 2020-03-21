use crate::assignments::get::parse_path;
use crate::db::rows_into;
use crate::error::HttpError;
use crate::handler::{redirect, render_template, HttpResult};
use crate::template::TEMPLATES;
use crate::State;
use actix_web::{web, HttpResponse};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(serde::Deserialize)]
pub struct ExerciseForm {
    description: String,
}

pub async fn insert(form: web::Form<ExerciseForm>, data: web::Data<State>) -> HttpResult {
    let client = data.db_pool.get().await?;
    let stmt = client
        .prepare("INSERT INTO exercise(description) VALUES($1)")
        .await?;
    client.execute(&stmt, &[&form.description]).await?;
    Ok(redirect("/"))
}

pub async fn page() -> HttpResult {
    render_template(&TEMPLATES, "exercise_form.html", &tera::Context::new())
}

pub async fn rename(
    form: web::Form<ExerciseForm>,
    data: web::Data<State>,
    path: web::Path<String>,
) -> HttpResult {
    let exercise_id = parse_path(&path.into_inner())?;
    let client = data.db_pool.get().await?;
    let stmt = client
        .prepare("UPDATE exercise SET description = $1 WHERE id = $2")
        .await?;
    client
        .execute(&stmt, &[&form.description, &exercise_id])
        .await
        .map_err(|_e| HttpError::NotFound(format!("Exercise {}", exercise_id)))?;
    Ok(redirect("/"))
}

pub async fn get_all_with_count(data: web::Data<State>) -> HttpResult {
    let client = data.db_pool.get().await?;
    let query = r#"
        SELECT count(a.id) as count, e.id, e.description
        FROM assignment a full outer JOIN exercise e ON a.exercise_id = e.id
        GROUP BY e.id, e.description
        ORDER BY e.description;
    "#;
    let rows = client.query(query, &[]).await?;
    let exercises = rows_into::<ExerciseCount>(rows);
    let mut context = tera::Context::new();
    context.insert("exercises", &exercises);
    render_template(&TEMPLATES, "exercise_list.html", &context)
}

pub async fn delete(data: web::Data<State>, path: web::Path<String>) -> HttpResult {
    let exercise_id = parse_path(&path.into_inner())?;
    dbg!(&exercise_id);
    let client = data.db_pool.get().await?;
    let stmt = client.prepare("DELETE FROM exercise where id = $1").await?;
    client
        .execute(&stmt, &[&exercise_id])
        .await
        .map_err(|_e| HttpError::NotFound(format!("Exercise {}", exercise_id)))?;
    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Clone, PostgresMapper, serde::Serialize)]
#[pg_mapper(table = "exercise")]
pub struct ExerciseCount {
    count: i64,
    id: i32,
    description: String,
}
