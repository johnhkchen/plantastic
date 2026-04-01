# T-016-01 Research: Scan Upload & Processing API

## Ticket Summary

Wire pt-scan to the API. PLY upload triggers async processing, stores outputs in S3, links to project. Zone editor loads plan view PNG as background.

---

## 1. pt-scan Public API

**File:** `crates/pt-scan/src/lib.rs`

Two-step pipeline:

```rust
pub fn process_scan(reader: impl Read, config: &ScanConfig) -> Result<PointCloud, ScanError>
pub fn generate_terrain(cloud: &PointCloud, config: &ExportConfig) -> Result<TerrainOutput, ScanError>
```

`TerrainOutput` contains three artifacts:
- `mesh_glb: Vec<u8>` — binary glTF 2.0 terrain mesh
- `plan_view_png: Vec<u8>` — orthographic top-down PNG
- `metadata: TerrainMetadata` — bbox, elevation range, counts, processing time

Both functions are CPU-bound, blocking, and deterministic. `process_scan` does PLY parsing + filtering + RANSAC. `generate_terrain` does triangulation + decimation + export. Combined processing time for a 630-point synthetic scan is ~100ms; real scans (millions of points) could take 5-30+ seconds.

**Error types:** `ScanError` — `InvalidPly`, `InsufficientPoints`, `NoGroundPlane`, `MeshGeneration`, `ExportError`, `Io`.

---

## 2. API Architecture

**File:** `crates/plantastic-api/src/`

### AppState

```rust
pub struct AppState {
    pub pool: PgPool,
}
```

Minimal — just the database pool. No S3 client, no job queue. Must be extended.

### Router

Routes merge into a single `Router<AppState>`:
- `health`, `projects`, `zones`, `materials`, `tiers`, `quotes`
- Middleware: `TraceLayer` + `CorsLayer::permissive()`
- All routes use `State(state): State<AppState>` for DB access

### Route Pattern (projects.rs)

1. Extract `TenantId` from `X-Tenant-Id` header (custom Axum extractor)
2. Validate tenant ownership via `verify_project_tenant()` helper
3. Call `pt_repo::project::*` for database operations
4. Return typed JSON responses with status codes
5. Errors auto-convert via `From<RepoError> for AppError`

### Existing Async Pattern (projects.rs:77-103)

Satellite baseline generation uses `tokio::task::spawn_blocking` inline:
```rust
let baseline_result = tokio::task::spawn_blocking(move || {
    builder.build(&addr)
}).await;
```
This blocks the handler until completion — not true async queuing. Fine for satellite (fast), not for scan processing (potentially slow).

### No File Upload Support

No multipart handling anywhere in the codebase. `axum` supports multipart via `axum::extract::Multipart` (built-in) or `axum-extra` for typed multipart.

---

## 3. Repository Layer

**File:** `crates/pt-repo/src/project.rs`

### ProjectRow

```rust
pub struct ProjectRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub scan_ref: Option<serde_json::Value>,  // JSONB — our target field
    pub baseline: Option<serde_json::Value>,
    // ...
}
```

### set_baseline Pattern (lines 111-126)

```rust
pub async fn set_baseline(pool: &PgPool, id: Uuid, baseline: &serde_json::Value) -> Result<(), RepoError>
```

Direct SQL update of JSONB field. **Need an identical `set_scan_ref` function.**

### Missing: No scan job tracking

No table or repository functions for scan job status. Need either:
- A dedicated `scan_jobs` table
- An in-memory job tracker (simpler for V1)

---

## 4. Database Schema

**File:** `migrations/002-create-projects.sql`

```sql
scan_ref JSONB,  -- Already exists in schema
```

The `scan_ref` column is already present as JSONB. No migration needed for storing scan artifact references. A new migration IS needed if we want a `scan_jobs` table for async job tracking.

---

## 5. Object Storage (S3)

### Current State: None

No AWS SDK dependency anywhere in the workspace. No S3 client, no bucket configuration.

### What's Needed

- `aws-sdk-s3` + `aws-config` as workspace dependencies
- S3 client in `AppState`
- Bucket name configuration via env var
- Upload functions for three artifacts (GLB, PNG, metadata JSON)
- Download/presign for serving plan view

### Key Consideration: Lambda Execution

On Lambda, credentials come from the execution role automatically via `aws-config`. Locally, use env vars or `~/.aws/credentials`. The `aws-config` crate handles both transparently.

---

## 6. Async Job Processing

### Requirements

- PLY upload returns 202 immediately with a job ID
- Processing runs asynchronously (not blocking the HTTP response)
- Status polling via `GET /projects/:id/scan/status`
- On completion: upload artifacts to S3, update project's `scan_ref`
- On failure: record error message

### Options Available

1. **Postgres-backed job queue** — new table, poll or LISTEN/NOTIFY
2. **In-process tokio::spawn** — simplest, jobs lost on Lambda cold restart
3. **SQS/external queue** — production-grade but heavy infrastructure

### Lambda Constraint

Lambda functions have a max execution time (15 min default). If the PLY upload handler returns 202 and spawns a background task, the Lambda invocation may freeze/terminate before processing completes. However, the response streaming mode (`RESPONSE_STREAM`) keeps the Lambda alive until the function completes, and `tokio::spawn` tasks run within the same runtime.

For V1/prototype: `tokio::spawn` with in-memory state tracking is viable. The scan processing runs in the same Lambda invocation as the upload. If the upload Lambda times out, the job fails — acceptable for V1.

---

## 7. Test Infrastructure

### Scenario S.1.1

Currently passes at `OneStar` (pure computation). Comment at line 198 explicitly says:
> "No API or UI integration yet — needs T-016-01 (upload API) for TwoStar."

This ticket should advance S.1.1 toward TwoStar by demonstrating the API upload → process → store → retrieve round-trip.

### Integration Test Pattern

`crates/plantastic-api/tests/common/mod.rs` provides:
- `test_pool()` — connects via DATABASE_URL
- `setup_test_db()` — runs migrations
- `create_test_tenant()` — inserts tenant row
- `test_router()` — builds router with real pool
- `send()` — sends JSON requests via `tower::ServiceExt::oneshot`

**Missing:** No `send_multipart()` helper for file uploads. Need to build multipart request bodies manually using `axum::body::Body` and content-type boundaries.

---

## 8. ProjectResponse DTO

**File:** `crates/plantastic-api/src/routes/projects.rs:31-41`

```rust
struct ProjectResponse {
    id, tenant_id, client_name, client_email, address, baseline, status, created_at, updated_at
}
```

**Notable omission:** `scan_ref` is not in `ProjectResponse`. The `From<ProjectRow>` impl skips it. This needs to be added so the frontend can see scan artifact URLs.

---

## 9. Constraints and Assumptions

1. **No existing S3 infrastructure** — this ticket introduces it for the first time
2. **No multipart handling** — first file upload endpoint
3. **No async job system** — first background processing endpoint
4. **scan_ref JSONB column exists** — no schema migration needed for that
5. **pt-scan is CPU-bound** — needs `spawn_blocking` to avoid starving async runtime
6. **Lambda cold starts** — S3 client initialization adds ~200ms (one-time per invocation)
7. **File size** — PLY files from SiteScape are typically 10-200MB; Lambda payload limit is 6MB for sync invocations but streaming mode allows larger payloads
8. **CLAUDE.md rule 3** — no mocking S3 across crate boundaries; integration tests should use real S3 or LocalStack (per feedback_no_minio.md: use R2 local or LocalStack, not MinIO)

---

## 10. Files Relevant to This Ticket

| File | Relevance |
|------|-----------|
| `crates/plantastic-api/src/state.rs` | Add S3 client + scan job tracker |
| `crates/plantastic-api/src/routes/mod.rs` | Register scan routes |
| `crates/plantastic-api/src/error.rs` | Add ScanError conversion |
| `crates/pt-repo/src/project.rs` | Add `set_scan_ref()` |
| `crates/pt-scan/src/lib.rs` | Public API (consumed, not modified) |
| `crates/plantastic-api/Cargo.toml` | Add aws-sdk-s3, pt-scan deps |
| `Cargo.toml` | Add workspace deps |
| `.env.example` | Add S3_BUCKET, AWS_REGION |
| `tests/scenarios/src/suites/site_assessment.rs` | Advance S.1.1 |
| `crates/plantastic-api/tests/` | Add scan upload integration test |
