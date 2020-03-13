mod api;
mod base64;
mod handlers;
mod state;
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{middleware, web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use failure::_core::time::Duration;
use futures::prelude::*;
use handlers::{
    auth::get_credentials, auth::my_basic_auth, get::get_assignments, get::get_result, get::index,
    get::version, post::add_submission,
};
mod rpc_conf;
use crate::rpc_conf::RpcEnvConfig;
use state::State;

async fn run() -> Result<(), failure::Error> {
    std::env::set_var("RUST_LOG", "info");
    let db_pool = db_lib::connect_migrate(db_lib::DB_URL)
        .await
        .expect("db connection err");
    let env_conf = envy::from_env::<RpcEnvConfig>()?;
    let state = State::new(env_conf, get_credentials(), db_pool);
    let c_state = state.clone();
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60 * 10));
        while let Some(_) = interval.next().await {
            c_state.pending_results.shrink_to_fit();
        }
    });
    env_logger::init();
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(Logger::default())
            .wrap(HttpAuthentication::basic(my_basic_auth))
            .route("/", web::get().to(index))
            .route("/version", web::get().to(version))
            .route("/assignments", web::get().to(get_assignments))
            .service(
                web::resource("/submission")
                    .data(
                        web::JsonConfig::default()
                            .error_handler(|err, _req| handlers::error::Error::Body(err).into()),
                    )
                    .route(web::post().to(add_submission)),
            )
            .service(
                web::resource("/result/{iliasId}")
                    .route(web::get().to(get_result))
                    .route(web::post().to(get_result)),
            )
            .wrap(
                Cors::new()
                    .allowed_methods(vec!["GET", "POST"])
                    .send_wildcard()
                    .finish(),
            )
            .data(state.clone())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;
    Ok(())
}
fn main() {
    if let Err(e) = actix_rt::System::new("test").block_on(run()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
