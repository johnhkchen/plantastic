//! Route assembly for the Plantastic API.

pub mod health;
pub mod materials;
pub mod projects;
pub mod proposals;
pub mod quotes;
pub mod scan;
pub mod scenes;
pub(crate) mod shared;
pub mod tiers;
pub mod zones;

use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

/// Build the complete API router with middleware.
pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(health::routes())
        .merge(projects::routes())
        .merge(zones::routes())
        .merge(materials::routes())
        .merge(tiers::routes())
        .merge(quotes::routes())
        .merge(proposals::routes())
        .merge(scenes::routes())
        .merge(scan::routes())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
