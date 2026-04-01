# T-016-01 Design: Scan Upload & Processing API

## Decision Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| S3 client | `aws-sdk-s3` + `aws-config` | Official AWS SDK; auto-credentials on Lambda |
| Multipart upload | `axum::extract::Multipart` | Built into axum, no extra dependency |
| Async processing | `tokio::spawn` + in-memory `DashMap` | Simplest V1; no new infrastructure |
| Job tracking | In-memory `ScanJobTracker` in AppState | Adequate for single-Lambda deployment |
| S3 key layout | `scans/{project_id}/{artifact}` | Simple, project-scoped |
| Plan view serving | Presigned S3 URL redirect | No proxy overhead |

---

## Option Analysis

### A. Object Storage Integration

**Option A1: aws-sdk-s3 (Official AWS SDK)**
- Pro: First-party, auto-credentials via IAM role on Lambda, async, well-maintained
- Pro: Compatible with S3, R2, LocalStack — all use S3 protocol
- Con: Adds ~2 crate deps to workspace
- **Chosen**

**Option A2: rusoto**
- Rejected: Maintenance mode, superseded by official SDK

**Option A3: Abstract storage trait**
- Rejected: Over-engineering for V1. Only one implementation needed (S3). Can abstract later.

### B. File Upload Handling

**Option B1: axum::extract::Multipart (built-in)**
- Pro: No additional dependency, standard multipart/form-data parsing
- Pro: Streaming — doesn't buffer entire file in memory
- Con: Manual field iteration
- **Chosen**

**Option B2: axum-typed-multipart**
- Rejected: Extra dependency for minimal gain. The upload is a single field.

### C. Async Job Processing

**Option C1: tokio::spawn + in-memory DashMap**
- Pro: Zero infrastructure. Job state lives in `Arc<DashMap<Uuid, ScanJobState>>`
- Pro: Works on Lambda — task runs in same invocation, completes before freeze
- Con: State lost on cold start (acceptable — completed jobs have scan_ref in DB)
- Con: No retry mechanism
- **Chosen for V1**

**Option C2: Postgres-backed job queue**
- Pro: Durable across restarts, queryable
- Con: New migration, polling logic, complexity
- Con: Lambda doesn't have a persistent worker polling the queue
- Deferred: Good option for production, but premature now

**Option C3: SQS + separate worker Lambda**
- Pro: Production-grade, decoupled
- Con: Significant infrastructure (SQS queue, separate Lambda, IAM)
- Deferred: Right answer for large files / high throughput

### D. Scan Job State Machine

```
Pending → Processing → Complete
                    → Failed
```

```rust
pub struct ScanJob {
    pub id: Uuid,
    pub project_id: Uuid,
    pub status: ScanJobStatus,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub enum ScanJobStatus {
    Pending,
    Processing,
    Complete,
    Failed,
}
```

On completion, the handler:
1. Uploads three artifacts to S3
2. Updates `projects.scan_ref` with a JSON object containing S3 keys
3. Sets job status to `Complete`

On failure, the handler:
1. Sets job status to `Failed` with error message
2. Logs the error

### E. scan_ref JSON Shape

```json
{
  "ply_key": "scans/{project_id}/raw.ply",
  "terrain_key": "scans/{project_id}/terrain.glb",
  "planview_key": "scans/{project_id}/planview.png",
  "metadata_key": "scans/{project_id}/metadata.json",
  "processed_at": "2026-03-31T..."
}
```

This matches the spec's expectation that `scan_ref` is a JSONB object with keys for all artifacts.

### F. Plan View Serving

**Option F1: Presigned URL redirect**
- `GET /projects/:id/planview` → 302 redirect to time-limited S3 presigned URL
- Pro: No proxy overhead, S3 handles bandwidth
- Con: URL expires (set to 1 hour, re-request as needed)
- **Chosen**

**Option F2: Stream through API**
- Pro: No presigning, consistent auth
- Con: Lambda bandwidth costs, memory overhead for large PNGs
- Rejected for V1

### G. PLY File Size and Lambda Limits

Lambda synchronous payload limit is 6MB. PLY files are 10-200MB. Two options:

**Option G1: S3 presigned upload + API trigger**
- Frontend gets a presigned PUT URL from API, uploads directly to S3, then calls API to trigger processing
- More complex but handles any file size
- **Chosen** — more robust

**Option G2: Multipart through API**
- Simple but hits Lambda payload limits for real scans
- Would work for dev/testing with small files

We'll implement G1 as the primary flow:
1. `POST /projects/:id/scan/upload-url` → returns presigned PUT URL + job ID
2. Frontend uploads PLY directly to S3
3. `POST /projects/:id/scan/process` → triggers processing of the uploaded PLY
4. `GET /projects/:id/scan/status` → poll for completion

However, for simplicity and to match the ticket's acceptance criteria (`POST /projects/:id/scan — accepts multipart PLY upload`), we'll ALSO support direct multipart upload for small files / testing. The multipart endpoint stores to S3 then triggers processing.

### H. Error Handling

New `AppError` variant not needed — existing variants cover all cases:
- Invalid PLY → `BadRequest`
- Project not found → `NotFound`
- S3 failure → `Internal`
- Processing failure → `Internal`

Add `From<ScanError> for AppError` and `From<aws_sdk_s3::...> for AppError` conversions.

---

## Rejected Alternatives

1. **Separate scan_jobs table** — Adds migration and query complexity for state that's ephemeral. The durable state is `scan_ref` in the projects table. Job status is transient.

2. **WebSocket for status updates** — Over-engineered for polling a simple status. SSE could work but polling is simpler and the ticket specifies a GET endpoint.

3. **Streaming the processing result** — The three artifacts need to land in S3 regardless. No benefit to streaming them to the client.

---

## Scenario Impact

- **S.1.1**: Should advance toward TwoStar by testing the upload → process → store → retrieve round-trip. Full TwoStar requires a running API + DB, so unit-level tests of the API route logic (without real Postgres/S3) won't qualify. The integration test (with `#[ignore]`) demonstrates readiness.

- **New milestone**: "Scan Upload API" — delivers the HTTP interface for scan ingestion. Unblocks frontend zone editor loading plan view as background.

---

## Dependencies Introduced

| Crate | Version | Purpose |
|-------|---------|---------|
| `aws-sdk-s3` | `1` | S3 operations |
| `aws-config` | `1` | Auto-credential loading |
| `dashmap` | `6` | Concurrent in-memory job map |
| `pt-scan` | path | Scan processing pipeline |
