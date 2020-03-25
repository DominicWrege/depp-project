use actix_web::web;

use crate::handlers::{
    error, get::get_assignments, get::get_result, get::index, get::status, get::version,
    post::add_submission,
};

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("", web::get().to(index)) // for testing only
            .route("/version", web::get().to(version))
            .route("/status", web::get().to(status))
            .route("/assignments", web::get().to(get_assignments))
            .service(
                web::resource("/submission")
                    .data(
                        web::JsonConfig::default()
                            .error_handler(|err, _req| error::Error::Body(err).into()),
                    )
                    .route(web::post().to(add_submission)),
            )
            .service(
                web::resource("/result/{iliasId}")
                    .route(web::get().to(get_result))
                    .route(web::post().to(get_result)),
            ),
    );
}
