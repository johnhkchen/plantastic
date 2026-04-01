# T-021-03 Structure: Lambda → Neon Connection Validation

## Files modified

### `crates/plantastic-api/src/routes/health.rs`
- Add `ready` handler: `GET /health/ready`
- Handler takes `State<AppState>` to access pool
- Runs `sqlx::query("SELECT 1").execute(&state.pool)` with 5s timeout
- Returns 200 `{"status":"ok","db":"ok","latency_ms":N}` on success
- Returns 503 `{"status":"degraded","db":"error","error":"..."}` on failure
- Register route in the existing `routes()` function

### `scripts/validate-lambda-neon.sh` (new)
- Bash script, same conventions as `verify-deploy.sh` and `verify-neon.sh`
- Takes `<api-url>` as argument
- Sections:
  1. Cold start: `curl` to `/health/ready` with timing
  2. Warm request: immediate second `curl`
  3. Concurrent: 10 parallel `curl` requests, collect all times
  4. Retry check: `aws logs filter-log-events` for retry WARN messages
- Pass/fail summary at end

### `justfile`
- Add recipe: `validate-neon-lambda url` → `./scripts/validate-lambda-neon.sh {{url}}`

### `docs/active/epics/E-008-deployment-pipeline.md`
- Infrastructure table: Railway row → Neon row
- Description paragraph: Railway references → Neon
- "What's needed" item 3: "Railway PostGIS" → "Neon PostGIS (done via T-021-01)"

### `docs/active/tickets/T-017-02-railway-s3-secrets.md`
- "Railway PostGIS" section heading → "Neon PostGIS"
- Acceptance criteria updated to reference Neon connection strings
- Verification section references Neon pooled endpoint

### `docs/active/work/T-021-03/results.md` (new, template)
- Template for recording validation results
- Sections: cold start timing, warm timing, concurrent timing, retry log, tuning notes
- Filled in manually by operator after running validation script

---

## Files NOT modified

- `crates/pt-repo/src/pool.rs` — no changes needed, retry logic is complete
- `crates/plantastic-api/src/main.rs` — no changes to startup flow
- `infra/sst.config.ts` — no infrastructure changes
- `crates/plantastic-api/src/lib.rs` — routes are composed in routes/mod.rs

---

## Module boundaries

### Health routes (`routes/health.rs`)
- Current: single `health()` handler, no state access
- After: two handlers — `health()` (liveness, stateless) and `ready()` (readiness, needs pool)
- The `routes()` function already returns `Router<AppState>`, so adding state-dependent
  handlers requires no signature change

### Public interface
- New endpoint: `GET /health/ready` — added to the public API surface
- No changes to existing endpoints
- No new crates or modules

---

## Ordering

1. Add `/health/ready` endpoint (code change)
2. Add unit test for the ready endpoint
3. Create validation script
4. Add justfile recipe
5. Update E-008 documentation
6. Update T-017-02 documentation
7. Create results template
8. Run `just check` to verify nothing breaks
