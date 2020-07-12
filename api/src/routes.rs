//! All routes an here defined to keep the ```main()``` cleaner.
use actix_web::web;

use crate::handlers::{
    get::get_assignments, get::get_result, get::index, get::status, get::version,
    post::add_submission,
};

/// Registers all routes
/// # Routes
/// **Note:** That all routes have a ```/api``` prefix.
/// See the openapi docs for more information.
/// * ```/assignments```
/// * ```/submission```
/// * ```/result/{iliasId}```
/// * ```/status```
/// * ```/version```
///
pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("", web::get().to(index)) // for testing only
            .route("/version", web::get().to(version))
            .route("/status", web::get().to(status))
            .route("/assignments", web::get().to(get_assignments))
            .service(web::resource("/submission").route(web::post().to(add_submission)))
            .service(
                web::resource("/result/{iliasId}")
                    .route(web::get().to(get_result))
                    .route(web::post().to(get_result)),
            ),
    );
}
