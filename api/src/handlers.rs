//! All HTTP handlers for each corresponding route and also error handling.

pub mod auth;
pub mod error;

/// Handler for all ```get``` request.
pub mod get;
/// Handler for all ```post``` request.
pub mod post;
