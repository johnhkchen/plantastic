# T-016-01 Plan: Implementation Steps

## Step 1: Add workspace dependencies

**Files:** `Cargo.toml`, `crates/plantastic-api/Cargo.toml`

Add `aws-sdk-s3`, `aws-config`, `dashmap` to workspace dependencies. Add `pt-scan`, `aws-sdk-s3`, `aws-config`, `dashmap` to plantastic-api deps.

**Verify:** `cargo check -p plantastic-api` compiles.

---

## Step 2: Add `set_scan_ref` to repository layer

**File:** `crates/pt-repo/src/project.rs`

Add `pub async fn set_scan_ref(pool, id, scan_ref: &serde_json::Value) -> Result<(), RepoError>`. Mirrors `set_baseline` exactly but updates the `scan_ref` column.

**Verify:** `cargo check -p pt-repo` compiles.

---

## Step 3: Create S3 helper module

**File:** `crates/plantastic-api/src/s3.rs`

Implement:
- `pub async fn create_s3_client() -> aws_sdk_s3::Client`
- `pub async fn upload_bytes(client, bucket, key, bytes, content_type) -> Result<()>`
- `pub async fn download_bytes(client, bucket, key) -> Result<Vec<u8>>`
- `pub async fn presigned_get_url(client, bucket, key, expires_secs) -> Result<String>`

Error type: wrap S3 SDK errors into a simple `S3Error` that converts to `AppError::Internal`.

**Verify:** `cargo check -p plantastic-api` compiles.

---

## Step 4: Create scan job tracker

**File:** `crates/plantastic-api/src/scan_job.rs`

Implement `ScanJobTracker` with `DashMap<Uuid, ScanJob>`. Methods: `create`, `get`, `get_by_project`, `set_processing`, `set_complete`, `set_failed`.

**Verify:** Unit test — create job, transition states, verify lookups.

---

## Step 5: Expand AppState and main.rs

**Files:** `state.rs`, `main.rs`, `lib.rs`

- Add `s3_client`, `s3_bucket`, `scan_jobs` to `AppState`
- In `main.rs`: initialize S3 client via `s3::create_s3_client()`, read `S3_BUCKET` env var, create `ScanJobTracker`
- In `lib.rs`: add `pub mod s3;` and `pub mod scan_job;`

**Verify:** `cargo check -p plantastic-api` compiles. Binary starts locally (S3 client init may warn if no AWS config, but shouldn't panic).

---

## Step 6: Add error conversions

**File:** `crates/plantastic-api/src/error.rs`

Add:
- `From<pt_scan::ScanError> for AppError` — `InvalidPly` → `BadRequest`, others → `Internal`
- S3 error conversion (from the S3Error type defined in step 3)

**Verify:** `cargo check -p plantastic-api` compiles.

---

## Step 7: Implement scan routes

**File:** `crates/plantastic-api/src/routes/scan.rs`

Implement three handlers:

### `upload_scan` (POST /projects/{id}/scan)
1. Extract tenant, verify project ownership
2. Extract multipart body, find "file" field, read PLY bytes
3. Generate S3 key: `scans/{project_id}/raw.ply`
4. Upload PLY to S3
5. Create scan job (Pending)
6. `tokio::spawn` the processing task (step 8)
7. Return 202 + `{ "job_id": "...", "status": "pending" }`

### `scan_status` (GET /projects/{id}/scan/status)
1. Extract tenant, verify project ownership
2. Look up latest job for project
3. If no job → return `{ "status": "none" }`
4. If complete → include artifact URLs from project's scan_ref
5. If failed → include error message

### `get_planview` (GET /projects/{id}/planview)
1. Extract tenant, verify project ownership
2. Read project's scan_ref
3. If no scan_ref → 404
4. Generate presigned URL for planview_key
5. Return 302 redirect

**Verify:** `cargo check -p plantastic-api` compiles.

---

## Step 8: Implement background processing function

**In:** `crates/plantastic-api/src/routes/scan.rs` (private function)

`async fn process_scan_job(...)`:
1. Set job status → Processing
2. Download PLY from S3
3. `spawn_blocking`: run `pt_scan::process_scan` + `generate_terrain`
4. Upload `terrain.glb`, `planview.png`, `metadata.json` to S3
5. Build scan_ref JSON object
6. Call `pt_repo::project::set_scan_ref`
7. Set job status → Complete
8. On any error: set job status → Failed with message

**Verify:** Compiles. Functional testing in step 11.

---

## Step 9: Register scan routes and update project response

**Files:** `routes/mod.rs`, `routes/projects.rs`

- Add `pub mod scan;` and `.merge(scan::routes())` to router
- Add `scan_ref: Option<serde_json::Value>` to `ProjectResponse` and its `From` impl

**Verify:** `cargo check -p plantastic-api` compiles.

---

## Step 10: Update .env.example and test infrastructure

**Files:** `.env.example`, `tests/common/mod.rs`

- Add `S3_BUCKET=plantastic-dev` and `AWS_REGION=us-west-2` to .env.example
- Update `test_router()` to construct full AppState with S3 client + job tracker
- Add `send_multipart()` helper for file upload tests

**Verify:** Existing tests still compile.

---

## Step 11: Write integration tests

**File:** `crates/plantastic-api/tests/scan_test.rs`

Tests (all `#[ignore]` — require Postgres + S3/LocalStack):
- `scan_upload_returns_202` — verify multipart upload accepted, job ID returned
- `scan_status_reflects_processing` — upload → poll → eventually complete
- `scan_planview_serves_redirect` — after completion, GET planview → 302
- `scan_ref_populated_after_processing` — GET project, verify scan_ref has keys
- `scan_upload_tenant_isolation` — tenant B cannot trigger scan on tenant A's project

**Verify:** `cargo test -p plantastic-api` (non-ignored tests pass). `cargo check --tests -p plantastic-api` (ignored tests compile).

---

## Step 12: Update scenario and claim milestone

**File:** `tests/scenarios/src/suites/site_assessment.rs`

Add a comment to S.1.1 noting TwoStar readiness path. Optionally add milestone in `progress.rs`.

**File:** `tests/scenarios/src/progress.rs`

Claim milestone: "Scan Upload API delivered by T-016-01".

**Verify:** `cargo run -p pt-scenarios` — no regressions, S.1.1 still passes at OneStar.

---

## Step 13: Format, lint, test

Run `just check` (fmt + lint + test + scenarios). Fix any issues.

---

## Testing Strategy

| Test Type | What | Where |
|-----------|------|-------|
| Unit | ScanJobTracker state transitions | `scan_job.rs` #[cfg(test)] |
| Unit | S3 key generation | `scan.rs` #[cfg(test)] |
| Integration | Full upload → process → status cycle | `tests/scan_test.rs` (#[ignore]) |
| Scenario | S.1.1 non-regression | `site_assessment.rs` |

### What doesn't need tests
- S3 upload/download (thin wrapper, tested via integration)
- Presigned URL generation (SDK function, tested via integration)
- Error conversions (trivial mapping, caught by integration tests)
