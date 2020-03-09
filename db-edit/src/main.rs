use crate::handler::{
    assignment_form, get_assignments_for_exercise, index, new_assignment, parse_template,
};
use actix_web::{web, App, HttpServer};
use anyhow::Context;
use deadpool_postgres::Pool;
use tera::Tera;
mod db;
mod handler;

// This struct represents state
#[derive(Clone)]
pub struct State {
    db_pool: Pool,
    temp: Tera,
}

#[actix_rt::main]
async fn main() -> Result<(), anyhow::Error> {
    let state = State {
        db_pool: db_lib::connect_migrate(db_lib::DB_URL).await?,
        temp: parse_template("templates/*.html"),
    };

    HttpServer::new(move || {
        App::new()
            .service(actix_files::Files::new("/templates", "./static").show_files_listing())
            .data(state.clone())
            .route("/", web::get().to(index))
            .route("/assignment_form", web::get().to(assignment_form))
            .route("/new_assignment", web::post().to(new_assignment))
            .route(
                "/exercise/{exercise_id}",
                web::get().to(get_assignments_for_exercise),
            )
    })
    .bind("127.0.0.1:8088")
    .context("Can not bind to port 8088")?
    .run()
    .await?;
    Ok(())
}
