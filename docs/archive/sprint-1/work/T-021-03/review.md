# T-021-03 Review: Lambda → Neon Connection Validation

## Summary

Added a database readiness endpoint, a connection validation script, and updated
documentation to complete the Railway → Neon migration paper trail.

## Files changed

### Modified
- `crates/plantastic-api/src/routes/health.rs` — added `/health/ready` readiness probe
- `justfile` — added `validate-neon-lambda` recipe
- `docs/active/epics/E-008-deployment-pipeline.md` — Railway → Neon throughout
- `docs/active/tickets/T-017-02-railway-s3-secrets.md` — Railway → Neon throughout

### Created
- `scripts/validate-lambda-neon.sh` — connection validation script (cold start, concurrency, idle)
- `docs/active/work/T-021-03/results.md` — template for recording validation results
- `docs/active/work/T-021-03/research.md` — RDSPI research artifact
- `docs/active/work/T-021-03/design.md` — RDSPI design artifact
- `docs/active/work/T-021-03/structure.md` — RDSPI structure artifact
- `docs/active/work/T-021-03/plan.md` — RDSPI plan artifact
- `docs/active/work/T-021-03/progress.md` — RDSPI progress artifact

## What was delivered

### `/health/ready` endpoint
- `GET /health/ready` pings the database with `SELECT 1`, 5-second timeout
- Returns `200 {"status":"ok","db":"ok","latency_ms":N}` on success
- Returns `503 {"status":"degraded","db":"error|timeout","error":"..."}` on failure
- Existing `/health` liveness probe unchanged (no DB dependency)
- Directly addresses AC: "health endpoint returns 200 even on cold start within acceptable time (< 5 seconds total)"

### Validation script
- `scripts/validate-lambda-neon.sh <api-url>` measures:
  1. Liveness (no DB) — baseline cold start timing
  2. Cold readiness — first DB hit including potential Neon wake
  3. Warm readiness — steady-state latency
  4. 10 concurrent cold starts — Lambda scaling behavior
  5. Sub-5-second acceptance criterion check
  6. Optional 10-minute idle recovery (`--idle` flag)
- Pass/fail summary, curl-based (no AWS CLI dependency for basic validation)

### Documentation updates
- E-008 epic fully reflects Neon: infrastructure table, description, what's-needed, success criteria
- T-017-02 ticket updated: Railway PostGIS → Neon PostGIS, pooled endpoint references, connection tuning cross-references T-020-02

## Test coverage

- **Unit tests:** No new unit tests added. The readiness endpoint is a thin wrapper
  (`SELECT 1` + timeout) that can only be meaningfully tested against a real database.
  Adding a mock would violate the project's "no mocks across crate boundaries" rule.
- **Integration test path:** The endpoint is exercised by `scripts/validate-lambda-neon.sh`
  against the real deployed Lambda + Neon stack. It's also reachable via the existing
  `crud_test.rs` integration tests (which test the full router), though they don't
  specifically target `/health/ready`.
- **Existing tests:** All workspace tests pass. No regressions.
- **Scenario dashboard:** 58.0/240.0 min (24.2%), 15/24 milestones. No change — this
  ticket is infrastructure validation, not a new customer-facing capability.

## Scenario dashboard (before/after)

- **Before:** 58.0 min / 240.0 min (24.2%), 8 pass, 0 fail, 9 not implemented
- **After:** 58.0 min / 240.0 min (24.2%), 8 pass, 0 fail, 9 not implemented
- No regression. This ticket doesn't advance a scenario — it validates infrastructure.

## Open concerns

1. **Results template is unfilled.** The actual connection timing data requires running
   `just validate-neon-lambda <url>` against the deployed stack. The script and template
   are ready; the operator needs to execute and record results.

2. **Idle recovery test is manual.** The `--idle` flag requires 10 minutes of wall-clock
   waiting. This is by design (no way to force Neon compute suspension from the client
   side), but it means the idle test won't run in any automated context.

3. **Health/ready doesn't report internal breakdown.** The endpoint reports total latency
   but not the Neon wake / TLS / query breakdown separately. This data is available in
   CloudWatch via pool.rs structured logs (`elapsed_ms`, retry attempts). The validation
   script directs the operator to check CloudWatch for this detail.

4. **No milestone claimed.** This ticket validates existing infrastructure rather than
   delivering a new capability. No milestone update in `progress.rs` is appropriate.

## Quality gate

```
just check  →  All gates passed.
  fmt-check  ✓
  lint       ✓
  test       ✓  (all workspace tests pass)
  scenarios  ✓  (58.0/240.0 min, no regression)
```
