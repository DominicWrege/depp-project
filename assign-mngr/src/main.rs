//! A small web app for managing all assignments. [Bootstrap](https://getbootstrap.com/) is used for teh CSS and the [Monaco editor](https://microsoft.github.io/monaco-editor/) to have a better editor then textarea.
//! [Actix-Web](https://github.com/actix/actix-web) is used for the http component with [tokio](https://github.com/tokio-rs/tokio) as the async runtime.
//! [Tera](https://github.com/Keats/tera) is used for rendering the HTMl templates.  
//! All assignments are stored in a PostgreSQL DB.
#[macro_use]
extern crate lazy_static;

use crate::auth::{login, login_page, logout};
use crate::auth_middleware::CheckLogin;
use crate::config::CookieConfig;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::cookie::SameSite;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use deadpool_postgres::Pool;
use failure::ResultExt;

mod assignments;
mod auth;
mod auth_middleware;
mod config;
mod db;
mod error;
mod exercises;
mod handler;
mod template;

#[derive(Clone)]
pub struct State {
    db_pool: Pool,
    pwd: Vec<u8>,
}

pub const PATH_PREFIX: &'static str = "/manage";
/// Real main function. Starting the middleware and global initialization the state.
async fn run() -> Result<(), failure::Error> {
    let config = config::get();
    let state = State {
        db_pool: db_lib::connect_migrate().await?,
        pwd: config.password,
    };

    init_logging();
    let host = format!("0.0.0.0:{}", config.port); // default port is 5000
    log::info!("Listening on http://{}", host);
    let cookie_conf = CookieConfig::new();
    HttpServer::new(move || {
        App::new()
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
            .wrap(CheckLogin)
            .wrap(Logger::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&cookie_conf.key())
                    .name("auth")
                    .max_age_time(chrono::Duration::days(3))
                    .same_site(SameSite::Strict)
                    .secure(cookie_conf.secure()),
            ))
            .service(
                web::scope(PATH_PREFIX)
                    .service(web::resource("").route(web::get().to(exercises::get_all_with_count))) // index
                    .service(
                        web::resource("/login")
                            .route(web::post().to(login))
                            .route(web::get().to(login_page)),
                    )
                    .service(web::resource("/logout").route(web::get().to(logout)))
                    .service(
                        web::resource("/assignment_form")
                            .route(web::get().to(assignments::new::get_form)),
                    )
                    .service(
                        web::resource("/new_assignment")
                            .route(web::post().to(assignments::new::insert)),
                    )
                    .service(
                        web::resource("/assignment/{uuid}")
                            .route(web::get().to(assignments::get::single_assignment))
                            .route(web::post().to(assignments::edit::update)),
                    )
                    .service(
                        web::scope("/assignment/file")
                            .route("/{uuid}", web::post().to(assignments::file::update_files))
                            .route("/{uuid}", web::get().to(assignments::file::download)),
                    )
                    .service(
                        web::resource("/exercise_form")
                            .route(web::get().to(exercises::page))
                            .route(web::post().to(exercises::insert)),
                    )
                    .service(
                        web::scope("/exercise")
                            .route("/rename/{exercise_id}", web::post().to(exercises::rename))
                            .route(
                                "/{exercise_id}",
                                web::get().to(assignments::get::all_assignments_for_exercise),
                            )
                            .route("/delete/{exercise_id}", web::delete().to(exercises::delete)),
                    ),
            )
            .data(state.clone())
    })
    .bind(host)
    .context(format!("Cant bind port {}", config.port))?
    .run()
    .await?;
    Ok(())
}
/// Fake main function calls only the ```run``` function.
fn main() {
    if let Err(e) = actix_rt::System::new("web-main").block_on(run()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn init_logging() {
    std::env::set_var("RUST_LOG", "assign_mngr=info,error,actix_web=info");
    env_logger::init();
}
