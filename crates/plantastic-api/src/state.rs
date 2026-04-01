//! Application state shared across all handlers.

use std::sync::Arc;

use sqlx::PgPool;

use crate::scan_job::ScanJobTracker;

/// Shared state passed to all Axum handlers via `State`.
#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub s3_client: aws_sdk_s3::Client,
    pub s3_bucket: String,
    pub scan_jobs: Arc<ScanJobTracker>,
}
