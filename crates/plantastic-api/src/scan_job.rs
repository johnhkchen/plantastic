//! In-memory scan job tracker for async processing status.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::Serialize;
use uuid::Uuid;

/// Thread-safe in-memory tracker for scan processing jobs.
#[derive(Debug)]
pub struct ScanJobTracker {
    jobs: DashMap<Uuid, ScanJob>,
    /// Maps project_id → latest job_id for quick lookup.
    by_project: DashMap<Uuid, Uuid>,
}

impl Default for ScanJobTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ScanJobTracker {
    pub fn new() -> Self {
        Self {
            jobs: DashMap::new(),
            by_project: DashMap::new(),
        }
    }

    /// Create a new pending scan job for a project.
    pub fn create(&self, project_id: Uuid) -> ScanJob {
        let job = ScanJob {
            id: Uuid::new_v4(),
            project_id,
            status: ScanJobStatus::Pending,
            error: None,
            created_at: Utc::now(),
            completed_at: None,
        };
        self.jobs.insert(job.id, job.clone());
        self.by_project.insert(project_id, job.id);
        job
    }

    /// Get a job by ID.
    pub fn get(&self, job_id: Uuid) -> Option<ScanJob> {
        self.jobs.get(&job_id).map(|r| r.clone())
    }

    /// Get the latest job for a project.
    pub fn get_by_project(&self, project_id: Uuid) -> Option<ScanJob> {
        let job_id = self.by_project.get(&project_id)?;
        self.jobs.get(&*job_id).map(|r| r.clone())
    }

    /// Transition a job to Processing.
    pub fn set_processing(&self, job_id: Uuid) {
        if let Some(mut job) = self.jobs.get_mut(&job_id) {
            job.status = ScanJobStatus::Processing;
        }
    }

    /// Transition a job to Complete.
    pub fn set_complete(&self, job_id: Uuid) {
        if let Some(mut job) = self.jobs.get_mut(&job_id) {
            job.status = ScanJobStatus::Complete;
            job.completed_at = Some(Utc::now());
        }
    }

    /// Transition a job to Failed with an error message.
    pub fn set_failed(&self, job_id: Uuid, error: String) {
        if let Some(mut job) = self.jobs.get_mut(&job_id) {
            job.status = ScanJobStatus::Failed;
            job.error = Some(error);
            job.completed_at = Some(Utc::now());
        }
    }
}

/// A scan processing job.
#[derive(Debug, Clone, Serialize)]
pub struct ScanJob {
    pub id: Uuid,
    pub project_id: Uuid,
    pub status: ScanJobStatus,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Scan job status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanJobStatus {
    Pending,
    Processing,
    Complete,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_lifecycle() {
        let tracker = ScanJobTracker::new();
        let project_id = Uuid::new_v4();

        // Create
        let job = tracker.create(project_id);
        assert_eq!(job.status, ScanJobStatus::Pending);
        assert!(job.error.is_none());

        // Lookup by ID
        let found = tracker.get(job.id).unwrap();
        assert_eq!(found.id, job.id);

        // Lookup by project
        let found = tracker.get_by_project(project_id).unwrap();
        assert_eq!(found.id, job.id);

        // Processing
        tracker.set_processing(job.id);
        let found = tracker.get(job.id).unwrap();
        assert_eq!(found.status, ScanJobStatus::Processing);

        // Complete
        tracker.set_complete(job.id);
        let found = tracker.get(job.id).unwrap();
        assert_eq!(found.status, ScanJobStatus::Complete);
        assert!(found.completed_at.is_some());
    }

    #[test]
    fn job_failure() {
        let tracker = ScanJobTracker::new();
        let project_id = Uuid::new_v4();

        let job = tracker.create(project_id);
        tracker.set_processing(job.id);
        tracker.set_failed(job.id, "invalid PLY format".to_string());

        let found = tracker.get(job.id).unwrap();
        assert_eq!(found.status, ScanJobStatus::Failed);
        assert_eq!(found.error.as_deref(), Some("invalid PLY format"));
        assert!(found.completed_at.is_some());
    }

    #[test]
    fn latest_job_per_project() {
        let tracker = ScanJobTracker::new();
        let project_id = Uuid::new_v4();

        let job1 = tracker.create(project_id);
        let job2 = tracker.create(project_id);

        // Latest job should be job2
        let found = tracker.get_by_project(project_id).unwrap();
        assert_eq!(found.id, job2.id);

        // job1 still accessible by ID
        let found = tracker.get(job1.id).unwrap();
        assert_eq!(found.id, job1.id);
    }

    #[test]
    fn unknown_project_returns_none() {
        let tracker = ScanJobTracker::new();
        assert!(tracker.get_by_project(Uuid::new_v4()).is_none());
        assert!(tracker.get(Uuid::new_v4()).is_none());
    }
}
