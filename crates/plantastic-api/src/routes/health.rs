//! Health check endpoints.
//!
//! - `GET /health` — liveness probe (no dependencies, always fast)
//! - `GET /health/ready` — readiness probe (pings database, reports latency)

use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use sqlx::Row;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/health/ready", get(ready))
}

async fn health() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Readiness probe: verifies database connectivity and reports latency.
///
/// Returns 200 if `SELECT 1` succeeds within 5 seconds, 503 otherwise.
/// Used by deployment validation and monitoring to distinguish "process alive"
/// from "process can serve requests."
async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    let start = std::time::Instant::now();

    let result = tokio::time::timeout(Duration::from_secs(5), async {
        sqlx::query("SELECT 1 AS one")
            .fetch_one(&state.pool)
            .await
            .map(|row| {
                let _: i32 = row.get("one");
            })
    })
    .await;

    #[allow(clippy::cast_possible_truncation)] // latency in ms always fits u64
    let latency_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            Json(json!({
                "status": "ok",
                "db": "ok",
                "latency_ms": latency_ms,
            })),
        ),
        Ok(Err(e)) => {
            tracing::warn!(error = %e, latency_ms, "Readiness check: database error");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "degraded",
                    "db": "error",
                    "error": e.to_string(),
                    "latency_ms": latency_ms,
                })),
            )
        }
        Err(_) => {
            tracing::warn!(latency_ms, "Readiness check: database timeout (5s)");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "degraded",
                    "db": "timeout",
                    "error": "database ping timed out after 5 seconds",
                    "latency_ms": latency_ms,
                })),
            )
        }
    }
}
