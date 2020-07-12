//! All HTTP handlers for each corresponding route and also error handling.

/// Implemented HTTP basic access authentication .
pub mod auth;
/// Error handling.
pub mod error;

/// Handler for all ```get``` request.
pub mod get;
/// Handler for all ```post``` request.
pub mod post;
