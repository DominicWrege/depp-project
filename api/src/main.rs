//! This is the REST API which uses `JSON`. It communicates via RPC to the RPC servers (Windows | Linux) by retrieving all assignments from the PostgreSQL Database.
//! [actix-web](https://github.com/actix/actix-web) is used for the http component with [tokio](https://github.com/tokio-rs/tokio) as the async runtime.
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
use handlers::{auth::get_credentials, auth::handle_basic_auth};
mod rpc_conf;
use state::State;
/// Real main function. Starting the middleware and global initialization the state.
async fn run() -> Result<(), failure::Error> {
    std::env::set_var("RUST_LOG", "api=info,error,warn,actix_web=info,warn");
    let db_pool = db_lib::connect_migrate().await?;
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
            .wrap(HttpAuthentication::basic(handle_basic_auth))
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

/// Fake main function calls only the ```run``` function.
fn main() {
    if let Err(e) = actix_rt::System::new("api-main").block_on(run()) {
        log::error!("{}", e);
        std::process::exit(1);
    }
}
