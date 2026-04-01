//! Async helpers for API-level (TwoStar+) scenario tests.
//!
//! These mirror the patterns in `crates/plantastic-api/tests/common/mod.rs`
//! but return `Result` instead of panicking, so callers can convert errors
//! into `ScenarioOutcome::Fail` or `ScenarioOutcome::Blocked`.

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

/// Create a database connection pool from DATABASE_URL.
pub async fn scenario_pool() -> Result<PgPool, String> {
    let url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL not set — cannot run API-level scenarios".to_string())?;
    pt_repo::create_pool(&url)
        .await
        .map_err(|e| format!("failed to create pool: {e}"))
}

/// Run all migrations against the database.
pub async fn setup_db(pool: &PgPool) -> Result<(), String> {
    let migration_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../migrations");
    let mut entries: Vec<_> = std::fs::read_dir(migration_dir)
        .map_err(|e| format!("cannot read migrations dir: {e}"))?
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.ends_with(".sql") && !name.ends_with(".down.sql")
        })
        .collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);

    for entry in entries {
        let sql = std::fs::read_to_string(entry.path())
            .map_err(|e| format!("failed to read {}: {e}", entry.path().display()))?;
        sqlx::raw_sql(&sql).execute(pool).await.map_err(|e| {
            format!(
                "migration {} failed: {e}",
                entry.file_name().to_string_lossy()
            )
        })?;
    }
    Ok(())
}

/// Create a test tenant, returning its UUID.
pub async fn create_tenant(pool: &PgPool, name: &str) -> Result<Uuid, String> {
    pt_repo::tenant::create(pool, name)
        .await
        .map_err(|e| format!("failed to create tenant: {e}"))
}

/// Build the API router backed by a real database pool.
pub async fn router(pool: PgPool) -> axum::Router {
    let s3_client = plantastic_api::s3::create_s3_client().await;
    let s3_bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "plantastic-test".to_string());
    let state = plantastic_api::AppState {
        pool,
        s3_client,
        s3_bucket,
        scan_jobs: std::sync::Arc::new(plantastic_api::scan_job::ScanJobTracker::new()),
    };
    plantastic_api::router(state)
}

/// Send a request to the router and return (status, body as JSON Value).
pub async fn api_call(
    app: &axum::Router,
    method: Method,
    uri: &str,
    tenant_id: Uuid,
    body: Option<Value>,
) -> Result<(StatusCode, Value), String> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Tenant-Id", tenant_id.to_string())
        .header("Content-Type", "application/json");

    let body = match body {
        Some(json) => {
            Body::from(serde_json::to_vec(&json).map_err(|e| format!("JSON serialize error: {e}"))?)
        }
        None => {
            builder = builder.header("Content-Length", "0");
            Body::empty()
        }
    };

    let request = builder
        .body(body)
        .map_err(|e| format!("request build error: {e}"))?;

    let response = app
        .clone()
        .oneshot(request)
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .map_err(|e| format!("body collect error: {e}"))?
        .to_bytes();

    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };

    Ok((status, json))
}
