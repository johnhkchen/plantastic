//! Plantastic API library — router, state, and error types.
//!
//! Separated from main.rs so integration tests can construct the full router.

mod error;
mod extract;
mod routes;
pub mod s3;
pub mod scan_job;
mod state;

pub use state::AppState;

/// Build the complete API router with middleware.
pub fn router(state: AppState) -> axum::Router {
    routes::router(state)
}
