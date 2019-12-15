mod api;
mod base64;
mod config;
mod crash_test;
mod exec;
mod fs_util;
mod handler;
mod state;
mod util;
use structopt::StructOpt;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use config::parse_config;
use failure::_core::time::Duration;
use futures::prelude::*;
use handler::{add_submission, get_assignments, get_result, index, version};
use state::State;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long)]
    config: std::path::PathBuf,
}

async fn run() -> Result<(), failure::Error> {
    let opt = Opt::from_args();
    let config = parse_config(&opt.config)?;
    std::env::set_var("RUST_LOG", "actix_web=info");
    // dbg!(&config);
    let state = State::new(config);
    let c_state = state.clone();
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60 * 10));
        while let Some(_) = interval.next().await {
            c_state.pending_results.shrink_to_fit()
        }
    });
    env_logger::init();
    HttpServer::new(move || {
        App::new()
            .route("/", web::get().to(index))
            .route("/version", web::get().to(version))
            .route("/assignments", web::get().to(get_assignments))
            .service(
                web::resource("/submission")
                    .data(
                        web::JsonConfig::default()
                            .error_handler(|err, _req| handler::Error::Body(err).into()),
                    )
                    .route(web::post().to(add_submission)),
            )
            .service(web::resource("/result/{iliasId}").route(web::get().to(get_result)))
            .wrap(
                Cors::new()
                    .allowed_methods(vec!["GET", "POST"])
                    .send_wildcard()
                    .finish(),
            )
            .data(state.clone())
    })
    .bind("0.0.0.0:8080")?
    .start()
    .await?;
    Ok(())
}
fn main() {
    if let Err(e) = actix_rt::System::new("test").block_on(run()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
