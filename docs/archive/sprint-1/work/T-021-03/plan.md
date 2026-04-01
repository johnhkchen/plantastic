# T-021-03 Plan: Lambda → Neon Connection Validation

## Step 1: Add `/health/ready` endpoint

**Files:** `crates/plantastic-api/src/routes/health.rs`

Add `ready()` handler:
- Extract `State(state)` from Axum
- Wrap `sqlx::query("SELECT 1").execute(&state.pool)` in `tokio::time::timeout(5s)`
- On success: return 200 with `{"status":"ok","db":"ok","latency_ms":N}`
- On timeout/error: return 503 with `{"status":"degraded","db":"error","error":"..."}`
- Register as `.route("/health/ready", get(ready))`

**Verify:** `cargo check -p plantastic-api`

## Step 2: Create validation script

**Files:** `scripts/validate-lambda-neon.sh` (new)

Script structure:
```
Usage: ./scripts/validate-lambda-neon.sh <api-url>
```

Checks:
1. **Cold start** — `curl -w` with timing variables to `/health/ready`
2. **Warm request** — immediate second request
3. **Concurrent cold starts** — 10 parallel curl processes via background jobs
4. **Summary** — pass/fail table with timing data

No `aws logs` dependency — keep it simple, curl-only. Operator can check
CloudWatch manually for retry logs.

**Verify:** `bash -n scripts/validate-lambda-neon.sh` (syntax check)

## Step 3: Add justfile recipe

**Files:** `justfile`

Add under Deployment section:
```
validate-neon-lambda url:
    ./scripts/validate-lambda-neon.sh {{url}}
```

**Verify:** `just --list | grep validate`

## Step 4: Update E-008 epic

**Files:** `docs/active/epics/E-008-deployment-pipeline.md`

Changes:
- Infrastructure table: "Railway (grandfathered $5/mo) | $5/mo" → "Neon (Launch plan, us-west-2) | Free tier*"
- Description: "Railway PostGIS" → "Neon PostGIS", remove "Railway manages the database" paragraph
- What's needed item 3: mark as done, reference T-021-01

**Verify:** Read the file, confirm consistency.

## Step 5: Update T-017-02 ticket

**Files:** `docs/active/tickets/T-017-02-railway-s3-secrets.md`

Changes:
- Context: "Connect the deployed Lambda to Railway PostGIS" → "...to Neon PostGIS"
- "Railway PostGIS" section → "Neon PostGIS" with updated criteria
- Verification section: reference Neon pooled endpoint

**Verify:** Read the file, confirm consistency.

## Step 6: Create results template

**Files:** `docs/active/work/T-021-03/results.md` (new)

Template with sections:
- Test environment (Lambda region, Neon project, date)
- Cold start timing breakdown
- Warm request timing
- Concurrent cold start results (10 invocations)
- Retry behavior observed
- Idle recovery (manual observation notes)
- Tuning applied
- Conclusion (pass/fail against AC)

## Step 7: Run quality gate

Run `just check` (fmt-check + lint + test + scenarios).
Fix any issues before proceeding to review.

---

## Testing strategy

- **Unit test:** The ready endpoint can be tested with axum's test utilities if
  we have a pool. However, this requires a live database. Since the endpoint is
  simple (SELECT 1 + timeout), and integration tests exist in pt-repo already,
  adding a unit test that mocks the pool would violate the "no mocks across crate
  boundaries" rule. Instead, the validation script IS the test — it hits the real
  deployed endpoint.

- **Existing tests:** Must not break. `just test` runs all workspace tests.
  The new code adds a route but no new unit tests that require a database.

- **Validation:** The bash script is the primary verification artifact.
  Results are recorded in results.md.
