# T-016-01 Progress: Scan Upload & Processing API

## Completed Steps

### Step 1: Add workspace dependencies
- Added `aws-sdk-s3`, `aws-config`, `dashmap` to workspace `Cargo.toml`
- Added `pt-scan`, `aws-sdk-s3`, `aws-config`, `dashmap` to `plantastic-api` Cargo.toml
- Enabled `multipart` feature on axum

### Step 2: Add `set_scan_ref` to repository
- Added `set_scan_ref(pool, id, scan_ref)` to `pt-repo/src/project.rs`
- Mirrors existing `set_baseline()` pattern exactly

### Step 3: Create S3 helper module
- `crates/plantastic-api/src/s3.rs`: `create_s3_client`, `upload_bytes`, `download_bytes`, `presigned_get_url`
- `S3Error` type with `Display` impl, converts to `AppError::Internal`

### Step 4: Create scan job tracker
- `crates/plantastic-api/src/scan_job.rs`: `ScanJobTracker` backed by `DashMap`
- `ScanJob` struct with status enum (Pending/Processing/Complete/Failed)
- Lookup by job ID and by project ID (latest job)
- 4 unit tests: lifecycle, failure, latest-per-project, unknown-returns-none

### Step 5: Expand AppState and main.rs
- `AppState` now holds `s3_client`, `s3_bucket`, `scan_jobs: Arc<ScanJobTracker>`
- `main.rs` initializes S3 client and reads `S3_BUCKET` env var

### Step 6: Add error conversions
- `From<pt_scan::ScanError> for AppError`: InvalidPly/InsufficientPoints → BadRequest, rest → Internal
- `From<S3Error> for AppError` → Internal

### Step 7-8: Implement scan routes + background processing
- `POST /projects/{id}/scan` — multipart upload → S3 → spawn processing → 202
- `GET /projects/{id}/scan/status` — poll job state + scan_ref on completion
- `GET /projects/{id}/planview` — presigned S3 redirect (302)
- Background `process_scan_job` via `tokio::spawn` + `spawn_blocking` for CPU work

### Step 9: Register routes and update ProjectResponse
- Added `scan_ref` field to `ProjectResponse` DTO
- Registered `scan::routes()` in router

### Step 10: Update .env.example and test infrastructure
- Added `S3_BUCKET`, `AWS_REGION` to `.env.example`
- Updated `tests/common/mod.rs` with `test_router_full()`, `send_multipart()`
- Updated `tests/scenarios/src/api_helpers.rs` for new AppState signature

### Step 11: Write integration tests
- 6 integration tests in `scan_test.rs` (all `#[ignore]`, require Postgres + S3)
- Coverage: upload→202, status polling, scan_ref in project, planview redirect, pre-upload status, tenant isolation

### Step 12: Claim milestone
- Added "Scan upload API" milestone in `progress.rs`

### Step 13: Quality gate
- `cargo fmt` — clean
- `cargo clippy -p plantastic-api -- -D warnings` — clean
- `cargo test --workspace` — all pass, 0 failures
- `cargo run -p pt-scenarios` ��� 8 pass, 0 fail, no regressions

## Deviations from Plan

None. All steps executed as planned.
