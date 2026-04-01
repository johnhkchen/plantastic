//! Application state shared across all handlers.

use std::fmt;
use std::sync::Arc;

use sqlx::PgPool;

use crate::scan_job::ScanJobTracker;

/// Shared state passed to all Axum handlers via `State`.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub s3_client: aws_sdk_s3::Client,
    pub s3_bucket: String,
    pub scan_jobs: Arc<ScanJobTracker>,
    pub proposal_generator: Arc<dyn pt_proposal::ProposalNarrativeGenerator>,
}

impl fmt::Debug for AppState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppState")
            .field("pool", &self.pool)
            .field("s3_bucket", &self.s3_bucket)
            .field("scan_jobs", &self.scan_jobs)
            .field("proposal_generator", &"<ProposalNarrativeGenerator>")
            .finish()
    }
}
