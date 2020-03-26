mod api;
mod base64;
mod handlers;
mod routes;
mod state;
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{middleware, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use failure::_core::time::Duration;
use futures::prelude::*;
use handlers::{auth::get_credentials, auth::my_basic_auth};
mod rpc_conf;
use crate::rpc_conf::RpcEnvConfig;
use state::State;

async fn run() -> Result<(), failure::Error> {
    std::env::set_var("RUST_LOG", "info");
    let db_pool = db_lib::connect_migrate().await.expect("db connection err");
    let env_conf = rpc_conf::get_config()?;
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
            .configure(routes::register_routes)
            .wrap(
                Cors::new()
                    .allowed_methods(vec!["GET", "POST"])
                    .send_wildcard()
                    .finish(),
            )
            .data(state.clone())
    })
    .bind("0.0.0.0:6000")?
    .run()
    .await?;
    Ok(())
}
fn main() {
    if let Err(e) = actix_rt::System::new("api-main").block_on(run()) {
        log::error!("{}", e);
        std::process::exit(1);
    }
}
