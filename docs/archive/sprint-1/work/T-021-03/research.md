# T-021-03 Research: Lambda → Neon Connection Validation

## Ticket scope

Validate Lambda → Neon connections under real conditions (cold starts, concurrency, idle),
update documentation to reference Neon instead of Railway.

Three deliverables: (1) validation script with timing, (2) resilience verification,
(3) documentation updates to E-008 and T-017-02.

---

## Current infrastructure state

### Connection pool (`crates/pt-repo/src/pool.rs`)

Fully implemented by T-020-02. Lambda-tuned defaults:
- `connect_timeout`: 15s (covers Neon cold-start 3-8s)
- `acquire_timeout`: 10s
- `idle_timeout`: 30s
- `max_connections`: 5, `min_connections`: 0
- Retry: 3 attempts, 500ms initial backoff, exponential
- Transient classification: `Io`, `Tls`, `PoolTimedOut` → retry; all else → fail fast
- Each attempt wrapped in `tokio::time::timeout`
- Logging: WARN on retry, INFO on success, ERROR on exhaustion

Connection string features parsed by sqlx:
- `sslmode=require` (Neon mandatory)
- `sslnegotiation=direct` (skips SSLRequest, saves ~50-100ms)
- `statement_cache_size=0` (required for PgBouncer transaction mode on pooled endpoint)

### Lambda setup (`crates/plantastic-api/src/main.rs`, `infra/sst.config.ts`)

- Runtime: `provided.al2023`, `arm64`, 256 MB, 30s timeout
- `RESPONSE_STREAM` invoke mode
- `DATABASE_URL` from SST Secret (SSM-backed)
- Auto-detects Lambda via `AWS_LAMBDA_RUNTIME_API` env var
- Tracing: JSON format in Lambda (CloudWatch), pretty locally
- Pool created at init (outside handler) — shared across invocations

### Health endpoint (`crates/plantastic-api/src/routes/health.rs`)

Returns `{"status":"ok","version":"..."}`. No database query — purely process health.
Does not measure database readiness. This is relevant: a health check that reports OK
while the database is still cold-starting could mask latency issues.

### Existing verification scripts

1. `scripts/verify-neon.sh` — validates Neon provisioning (T-021-01):
   direct/pooled connectivity, PostGIS, tables, spatial roundtrip. Uses `psql`.
2. `scripts/verify-deploy.sh` — validates deployed Lambda:
   health check (with timing), create/fetch/delete project round-trip.

Neither script measures the specific cold-start timing breakdown this ticket requires.

### SST config (`infra/sst.config.ts`)

`DatabaseUrl` is an `sst.Secret` — resolved from SSM at deploy time.
Currently references the Neon pooled URL (set via `sst secret set`).
Lambda region: `us-west-2`, same as Neon project.

---

## What needs to happen

### 1. Connection validation script

Need a script that measures and reports:
- Lambda init (Rust binary startup) — time from function invoke to first line of user code
- Neon compute wake — time for suspended compute to become responsive
- TLS handshake — connection establishment over Neon's pooled endpoint
- First query — `SELECT 1` latency after connection is established

Approach: Instrument the existing Lambda rather than a standalone binary.
The pool.rs already logs `elapsed_ms` on connection. CloudWatch logs capture
Lambda INIT duration. The validation script invokes Lambda and parses results.

### 2. Concurrency and idle testing

- 10-minute idle: invoke Lambda, wait 10 min, invoke again. Neon free/launch tier
  suspends compute after 5 min idle by default. Second invoke hits cold Neon.
- 10 concurrent cold starts: use `aws lambda invoke` with concurrent async calls.
  Each gets its own Lambda instance → each creates its own pool → 10 simultaneous
  Neon connections.

### 3. Resilience verification

- Retry logic: already in pool.rs. Need to confirm it fires during Neon cold-start.
  CloudWatch logs should show WARN-level retry messages.
- No-hang guarantee: `tokio::time::timeout` wrapping each attempt ensures hangs
  are caught. The documented sqlx/tokio-postgres hang is mitigated by this.
- Health endpoint < 5s: currently doesn't hit DB. If we want health to reflect
  DB readiness, we need to add a DB ping. But the AC says "health endpoint returns
  200 even on cold start within acceptable time (< 5 seconds total)" — meaning
  the full request (including Lambda init + DB connect) should be < 5s.

### 4. Documentation updates

- E-008 epic: Railway → Neon in infrastructure table, description, and what's-needed
- T-017-02: Railway-specific acceptance criteria → Neon equivalents

---

## Existing patterns to follow

- Bash scripts in `scripts/` with `set -euo pipefail`, usage help, pass/fail counters
- `just` recipes for common operations
- SST for infrastructure, Doppler/SSM for secrets
- CloudWatch structured JSON logs from Lambda
- Integration tests gated behind `feature = "integration"`

## Constraints

- Lambda is already deployed (T-017-01 done)
- Neon is provisioned (T-021-01 done)
- Connection hardening is in place (T-020-02 done)
- We don't have access to invoke Lambda from CI — this is a manual validation ticket
- Results are documented in work artifacts, not automated in CI (CI Neon branching is T-021-02)

## Risks

- Neon free/launch tier suspend timeout may differ from documentation
- 10 concurrent Lambda invocations may hit AWS concurrency limits if not configured
- Connection timing varies significantly between warm Neon and cold Neon
- The health endpoint currently doesn't touch the DB, so "< 5 seconds total" needs
  clarification: is it health-only or first-DB-query?
