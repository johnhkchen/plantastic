//! Shared test infrastructure for plantastic-api integration tests.
#![allow(dead_code)]

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use sqlx::PgPool;
use tower::ServiceExt;

use plantastic_api::scan_job::ScanJobTracker;

/// Connect to the test database.
pub async fn test_pool() -> PgPool {
    let url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for integration tests");
    pt_repo::create_pool(&url)
        .await
        .expect("failed to create test pool")
}

/// Run all migrations.
pub async fn setup_test_db(pool: &PgPool) {
    let migration_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../migrations");
    let mut entries: Vec<_> = std::fs::read_dir(migration_dir)
        .expect("migrations directory should exist")
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.ends_with(".sql") && !name.ends_with(".down.sql")
        })
        .collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);

    for entry in entries {
        let sql = std::fs::read_to_string(entry.path())
            .unwrap_or_else(|_| panic!("failed to read {}", entry.path().display()));
        sqlx::raw_sql(&sql).execute(pool).await.unwrap_or_else(|e| {
            panic!(
                "migration {} failed: {e}",
                entry.file_name().to_string_lossy()
            )
        });
    }
}

/// Create a test tenant, returning its UUID.
pub async fn create_test_tenant(pool: &PgPool, name: &str) -> uuid::Uuid {
    pt_repo::tenant::create(pool, name)
        .await
        .expect("failed to create test tenant")
}

/// Create an S3 client for tests.
///
/// Uses real AWS config from environment (e.g., LocalStack endpoint).
pub async fn test_s3_client() -> aws_sdk_s3::Client {
    plantastic_api::s3::create_s3_client().await
}

/// Build the full API router with a real database pool and S3 client.
pub async fn test_router_full(pool: PgPool) -> axum::Router {
    let s3_client = test_s3_client().await;
    let s3_bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "plantastic-test".to_string());
    let state = plantastic_api::AppState {
        pool,
        s3_client,
        s3_bucket,
        scan_jobs: Arc::new(ScanJobTracker::new()),
        proposal_generator: Arc::new(pt_proposal::MockProposalGenerator),
    };
    plantastic_api::router(state)
}

/// Build the full API router (backwards-compatible for existing tests).
pub fn test_router(pool: PgPool) -> axum::Router {
    // For tests that don't need S3, create a minimal client.
    // S3 operations will fail but non-scan routes work fine.
    let rt = tokio::runtime::Handle::current();
    let s3_client = rt.block_on(plantastic_api::s3::create_s3_client());
    let state = plantastic_api::AppState {
        pool,
        s3_client,
        s3_bucket: "plantastic-test".to_string(),
        scan_jobs: Arc::new(ScanJobTracker::new()),
        proposal_generator: Arc::new(pt_proposal::MockProposalGenerator),
    };
    plantastic_api::router(state)
}

/// Send a request to the router and return (status, body as Value).
pub async fn send(
    app: &axum::Router,
    method: Method,
    uri: &str,
    tenant_id: uuid::Uuid,
    body: Option<serde_json::Value>,
) -> (StatusCode, serde_json::Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Tenant-Id", tenant_id.to_string())
        .header("Content-Type", "application/json");

    let body = match body {
        Some(json) => Body::from(serde_json::to_vec(&json).unwrap()),
        None => {
            builder = builder.header("Content-Length", "0");
            Body::empty()
        }
    };

    let request = builder.body(body).unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();

    let json = if bytes.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
    };

    (status, json)
}

/// Build a multipart/form-data body with a single "file" field.
pub fn build_multipart_body(file_bytes: &[u8]) -> (String, Vec<u8>) {
    let boundary = "----TestBoundary1234567890";
    let mut body = Vec::new();

    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"file\"; filename=\"scan.ply\"\r\n",
    );
    body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
    body.extend_from_slice(file_bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let content_type = format!("multipart/form-data; boundary={boundary}");
    (content_type, body)
}

/// Send a multipart file upload request.
pub async fn send_multipart(
    app: &axum::Router,
    method: Method,
    uri: &str,
    tenant_id: uuid::Uuid,
    file_bytes: &[u8],
) -> (StatusCode, serde_json::Value) {
    let (content_type, body) = build_multipart_body(file_bytes);

    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Tenant-Id", tenant_id.to_string())
        .header("Content-Type", content_type)
        .body(Body::from(body))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();

    let json = if bytes.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
    };

    (status, json)
}
