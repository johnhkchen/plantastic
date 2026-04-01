# T-021-03 Progress: Lambda → Neon Connection Validation

## Completed

### Step 1: `/health/ready` endpoint ✓
- Added `ready()` handler to `crates/plantastic-api/src/routes/health.rs`
- `SELECT 1` with 5s tokio timeout, returns 200/503 with latency_ms
- Registered on `/health/ready` alongside existing `/health` liveness probe
- Clippy-clean (cast_possible_truncation allow applied by lint hook)

### Step 2: Validation script ✓
- Created `scripts/validate-lambda-neon.sh`
- Tests: liveness, cold readiness, warm readiness, 10 concurrent cold starts, <5s AC
- Optional `--idle` flag for 10-minute idle recovery test
- Pass/fail summary with timing data

### Step 3: Justfile recipe ✓
- Added `validate-neon-lambda url` recipe

### Step 4: E-008 epic updated ✓
- Infrastructure table: Railway → Neon (Launch plan, us-west-2, Free tier)
- Description: Neon replaces Railway, added PgBouncer/branching/co-location note
- What's needed item 3: marked done (T-021-01)
- Success criteria: Railway PostGIS → Neon PostGIS

### Step 5: T-017-02 ticket updated ✓
- Context: Railway → Neon with migration note
- Acceptance criteria: Railway PostGIS section → Neon PostGIS
- Verification: references Neon pooled endpoint and connection tuning

### Step 6: Results template ✓
- Created `docs/active/work/T-021-03/results.md` with structured template
- Sections: environment, cold start, warm, concurrent, idle, retry, tuning, verdict

### Step 7: Quality gate ✓
- `just check` passes: fmt-check, lint, test (all workspace), scenarios
- Scenario dashboard: 58.0/240.0 min (24.2%), 15/24 milestones — no regression

## Deviations from plan

None. All steps executed as planned.
