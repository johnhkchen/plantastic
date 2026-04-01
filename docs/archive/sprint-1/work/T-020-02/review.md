# T-020-02 Review: Connection Hardening

## Summary

Hardened pt-repo's database connection pool for Neon serverless Postgres on AWS Lambda. Added configurable timeouts, exponential-backoff retry on transient failures, and documented Neon connection string patterns.

## Files Changed

| File | Change |
|------|--------|
| `crates/pt-repo/Cargo.toml` | Added `tracing` workspace dependency |
| `crates/pt-repo/src/pool.rs` | Rewritten: PoolConfig, retry logic, timeout wrapper, error classification |
| `crates/pt-repo/src/lib.rs` | Added `create_pool_with_config` and `PoolConfig` to public exports |
| `crates/pt-repo/tests/pool_test.rs` | New: 4 integration tests for pool creation |
| `.env.example` | Added Neon direct and pooled connection string examples |
| `tests/scenarios/src/progress.rs` | Claimed milestone for T-020-02 |

## What Was Delivered

### PoolConfig struct
- `connect_timeout` (15s default): wraps each attempt in `tokio::time::timeout`
- `acquire_timeout` (10s default): pool-level wait for available connection
- `idle_timeout` (30s): Lambda freeze-friendly
- `max_connections` (5) / `min_connections` (0): Lambda-appropriate pool sizing
- `max_retries` (3): number of connection attempts
- `initial_backoff` (500ms): doubles each retry (500ms → 1s → 2s)

### Retry logic
- `is_transient()` classifies sqlx errors: Io, Tls, PoolTimedOut are retriable
- Configuration and auth errors fail immediately (no wasted retries)
- Each attempt wrapped in `tokio::time::timeout` for hard deadline
- Timeout expiry converted to `sqlx::Error::Io(TimedOut)` for consistent handling
- Logging: WARN per retry, INFO on success, ERROR when all attempts fail

### Backwards compatibility
- `create_pool(&str)` signature preserved — delegates to `create_pool_with_config` with defaults
- API call site (`plantastic-api/src/main.rs`) requires no changes
- All existing 19 CRUD integration tests unaffected (they use `#[sqlx::test]`, not `create_pool()`)

### Neon connection string support
- `sslnegotiation=direct`: parsed by sqlx from URL, no code needed
- `statement_cache_size=0`: parsed by sqlx from URL, required for PgBouncer transaction mode
- `-pooler` hostname: just DNS, no special handling
- All documented in `.env.example` and pool.rs module docs

## Test Coverage

| Test | Type | What It Verifies |
|------|------|------------------|
| `default_config_matches_ticket_spec` | Unit | PoolConfig defaults match acceptance criteria |
| `io_error_is_transient` | Unit | I/O errors classified as retriable |
| `pool_timeout_is_transient` | Unit | Pool timeout classified as retriable |
| `config_error_is_not_transient` | Unit | Config errors not retried |
| `row_not_found_is_not_transient` | Unit | Query errors not retried |
| `pool_closed_is_not_transient` | Unit | Pool shutdown not retried |
| `pool_connects_with_defaults` | Integration | Default config works against local PG |
| `pool_connects_with_custom_config` | Integration | Custom config honored |
| `pool_rejects_invalid_url` | Integration | Bad URLs fail without retry |
| `pool_respects_max_connections` | Integration | max_connections propagated to pool |

6 unit tests (no database), 4 integration tests (feature-gated, require Docker PG).

## Scenario Dashboard

**Before:** 58.0 min / 240.0 min (24.2%), 14/23 milestones
**After:** 58.0 min / 240.0 min (24.2%), 15/24 milestones

No scenario regression. Milestone claimed: "pt-repo: connection hardening (timeouts, retry, Neon support)" unlocking S.INFRA.1 and S.INFRA.2. This is infrastructure work — it doesn't directly flip scenarios but unblocks reliable Neon connections for production Lambda deployments.

## Design Deviation

`PgConnectOptions` in sqlx 0.8 does not expose a `connect_timeout()` method (contrary to my initial assumption in design.md). Solved by wrapping `PgPoolOptions::connect_with()` in `tokio::time::timeout()`, which is actually better — it covers the entire connection establishment (TCP + TLS + auth), not just the TCP socket.

## Open Concerns

1. **No live Neon testing:** All tests run against local Docker Postgres. Neon cold-start behavior (3-8s) cannot be verified without a real Neon instance. T-021-01 (Neon provisioning) must be completed first.
2. **Worst-case latency:** With 15s connect_timeout × 3 retries + backoff, worst case is ~48.5s. Lambda's default 30s timeout would kill the function. The Lambda function timeout (configured in SST at 30s per T-017-01) should be increased to 60s, or connect_timeout reduced via PoolConfig for Lambda environments.
3. **PgPoolOptions::clone():** Each retry clones the pool options. This is cheap (just config values) but worth noting.

## Quality Gate

`just check` passes: format, clippy strict, all workspace tests, scenario dashboard.
