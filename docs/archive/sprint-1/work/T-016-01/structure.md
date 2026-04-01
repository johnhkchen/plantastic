# T-016-01 Structure: File-Level Changes

## New Files

### `crates/plantastic-api/src/routes/scan.rs`
Scan upload and status routes. Public function `routes() -> Router<AppState>`.

**Routes:**
- `POST /projects/{id}/scan` — multipart PLY upload → store in S3 → spawn processing → 202 + job ID
- `GET /projects/{id}/scan/status` — return job status (pending/processing/complete/failed) + output URLs
- `GET /projects/{id}/planview` — redirect to presigned S3 URL for plan view PNG

**Handlers:**
- `upload_scan(tenant, state, project_id, multipart)` → validates tenant, extracts PLY bytes from multipart, uploads raw PLY to S3, creates job, spawns processing task, returns 202 + `{job_id}`
- `scan_status(tenant, state, project_id)` → returns job state; if complete, includes artifact URLs from project's scan_ref
- `get_planview(tenant, state, project_id)` → reads scan_ref, generates presigned URL for planview_key, returns 302 redirect

**Internal:**
- `process_scan_job(pool, s3, bucket, job_tracker, project_id, ply_key)` — async function spawned via `tokio::spawn`. Downloads PLY from S3, runs `pt_scan::process_scan` + `generate_terrain` in `spawn_blocking`, uploads artifacts to S3, updates project's scan_ref, marks job complete/failed.

### `crates/plantastic-api/src/scan_job.rs`
In-memory scan job tracker.

**Types:**
```rust
pub struct ScanJobTracker { inner: DashMap<Uuid, ScanJob> }
pub struct ScanJob { id, project_id, status, error, created_at, completed_at }
pub enum ScanJobStatus { Pending, Processing, Complete, Failed }
```

**Methods:**
- `new()` — empty tracker
- `create(project_id) -> ScanJob` — insert new Pending job
- `get(job_id) -> Option<ScanJob>` — lookup
- `get_by_project(project_id) -> Option<ScanJob>` — latest job for project
- `set_processing(job_id)`
- `set_complete(job_id)`
- `set_failed(job_id, error: String)`

### `crates/plantastic-api/src/s3.rs`
S3 helper functions. Thin wrappers around aws-sdk-s3.

**Functions:**
- `create_s3_client() -> aws_sdk_s3::Client` — loads config from environment
- `upload_bytes(client, bucket, key, bytes, content_type) -> Result<(), S3Error>`
- `presigned_get_url(client, bucket, key, expires_secs) -> Result<String, S3Error>`
- `download_bytes(client, bucket, key) -> Result<Vec<u8>, S3Error>`

---

## Modified Files

### `crates/plantastic-api/src/state.rs`
Add S3 client, bucket name, and job tracker to AppState.

```rust
pub struct AppState {
    pub pool: PgPool,
    pub s3_client: aws_sdk_s3::Client,
    pub s3_bucket: String,
    pub scan_jobs: Arc<ScanJobTracker>,
}
```

### `crates/plantastic-api/src/main.rs`
Initialize S3 client and scan job tracker in main, pass to AppState.

### `crates/plantastic-api/src/lib.rs`
Add `pub mod scan_job;` and `pub mod s3;` module declarations. Re-export `ScanJobTracker`.

### `crates/plantastic-api/src/routes/mod.rs`
Add `pub mod scan;` and merge `scan::routes()` into the router.

### `crates/plantastic-api/src/error.rs`
Add `From<pt_scan::ScanError> for AppError` conversion. Add generic S3 error handling.

### `crates/plantastic-api/src/routes/projects.rs`
Add `scan_ref` field to `ProjectResponse` DTO and its `From<ProjectRow>` impl.

### `crates/plantastic-api/Cargo.toml`
Add dependencies: `aws-sdk-s3`, `aws-config`, `dashmap`, `pt-scan`.

### `Cargo.toml` (workspace root)
Add workspace dependencies: `aws-sdk-s3`, `aws-config`, `dashmap`.

### `crates/pt-repo/src/project.rs`
Add `set_scan_ref(pool, id, scan_ref: &serde_json::Value) -> Result<(), RepoError>` — mirrors `set_baseline`.

### `.env.example`
Add `S3_BUCKET` and `AWS_REGION` entries.

### `crates/plantastic-api/tests/common/mod.rs`
Update `test_router()` to accept the expanded AppState (S3 client + job tracker). Add a `mock_s3_state()` or `test_state()` helper.

### `crates/plantastic-api/tests/crud_test.rs`
Update existing test helpers to work with new AppState signature (add default S3 + job tracker).

---

## New Test Files

### `crates/plantastic-api/tests/scan_test.rs`
Integration tests for scan upload routes. All `#[ignore]` (require Postgres + S3).

- `scan_upload_and_status()` — upload PLY, poll status, verify scan_ref populated
- `scan_upload_invalid_ply()` — upload garbage data, expect meaningful error
- `scan_planview_redirect()` — upload + process, then GET planview, expect 302
- `scan_status_no_scan()` — GET status before any upload, expect appropriate response
- `scan_tenant_isolation()` — tenant A's scan not visible to tenant B

---

## Module Boundaries

```
plantastic-api (lib)
├── state.rs          — AppState (pool + s3 + jobs)
├── error.rs          — AppError + From impls
├── extract.rs        — TenantId extractor (unchanged)
├── s3.rs             — S3 helpers (NEW)
├── scan_job.rs       — In-memory job tracker (NEW)
└── routes/
    ├── mod.rs        — router assembly
    ├── health.rs     — unchanged
    ├── projects.rs   — add scan_ref to response
    ├── zones.rs      — unchanged
    ├── materials.rs  — unchanged
    ├── tiers.rs      — unchanged
    ├── quotes.rs     — unchanged
    └── scan.rs       — scan upload/status/planview (NEW)
```

### Dependency Flow

```
routes/scan.rs
  → scan_job.rs (job tracking)
  → s3.rs (upload/download/presign)
  → pt_scan (process_scan, generate_terrain)
  → pt_repo::project (set_scan_ref, get_by_id)
```

### Public Interface of scan Routes

```
POST   /projects/{id}/scan          → 202 { job_id, status: "pending" }
GET    /projects/{id}/scan/status   → 200 { job_id, status, error?, outputs? }
GET    /projects/{id}/planview      → 302 redirect to presigned S3 URL
```

---

## S3 Key Layout

```
{bucket}/
  scans/
    {project_id}/
      raw.ply           — uploaded PLY file
      terrain.glb       — generated terrain mesh
      planview.png      — orthographic plan view
      metadata.json     — processing metadata
```

---

## Ordering of Changes

1. Workspace + API Cargo.toml deps (no code changes, just build)
2. `s3.rs` — standalone helper module
3. `scan_job.rs` — standalone job tracker
4. `state.rs` + `main.rs` — expand AppState
5. `pt-repo/project.rs` — add `set_scan_ref`
6. `error.rs` — add From conversions
7. `routes/scan.rs` — the main routes
8. `routes/mod.rs` — register routes
9. `routes/projects.rs` — add scan_ref to response
10. `.env.example` — document new env vars
11. Tests
