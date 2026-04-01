# T-020-02 Plan: Connection Hardening

## Step 1: Add tracing dependency

**File:** `crates/pt-repo/Cargo.toml`
**Change:** Add `tracing.workspace = true` to `[dependencies]`
**Verify:** `cargo check -p pt-repo`

## Step 2: Rewrite pool.rs

**File:** `crates/pt-repo/src/pool.rs`
**Changes:**
1. Add `PoolConfig` struct with `Debug` derive and `Default` impl:
   - `connect_timeout: Duration` — default 15s
   - `acquire_timeout: Duration` — default 10s
   - `idle_timeout: Duration` — default 30s
   - `max_connections: u32` — default 5
   - `min_connections: u32` — default 0
   - `max_retries: u32` — default 3
   - `initial_backoff: Duration` — default 500ms

2. Add `is_transient(err: &sqlx::Error) -> bool`:
   - `Io(_)` → true
   - `Tls(_)` → true
   - `PoolTimedOut` → true
   - All others → false

3. Add `connect_with_retry()`:
   - Loop up to `max_retries` times
   - On transient error: log at WARN (attempt N/M, error, backoff), sleep backoff, double backoff
   - On permanent error: return immediately
   - On success: log at INFO (connected, attempts, elapsed)

4. Add `create_pool_with_config(database_url, config)`:
   - Parse URL into `PgConnectOptions` via `from_str()`
   - Apply `connect_timeout` from config
   - Build `PgPoolOptions` with config values
   - Call `connect_with_retry()`
   - Map error to `RepoError::Database`

5. Refactor `create_pool(database_url)`:
   - Delegate to `create_pool_with_config(database_url, &PoolConfig::default())`

**Verify:** `cargo check -p pt-repo`

## Step 3: Update lib.rs exports

**File:** `crates/pt-repo/src/lib.rs`
**Change:** Add `pub use pool::PoolConfig;`
**Verify:** `cargo check -p pt-repo`

## Step 4: Add pool integration tests

**File:** `crates/pt-repo/tests/pool_test.rs` (new)
**Tests:**
1. `pool_connects_with_defaults` — `create_pool()` with local Docker Postgres URL, verify pool works by executing `SELECT 1`
2. `pool_connects_with_custom_config` — `create_pool_with_config()` with short timeouts, verify connection
3. `pool_rejects_invalid_url` — bad URL returns error immediately (no retry on config errors)

All feature-gated with `#![cfg(feature = "integration")]`.

**Verify:** `cargo test -p pt-repo --features integration` (requires Docker Postgres)

## Step 5: Add unit tests for is_transient

In `pool.rs`, add `#[cfg(test)] mod tests` with:
1. `io_error_is_transient` — verify `Io` variant classified as transient
2. `config_error_is_not_transient` — verify `Configuration` classified as permanent

**Verify:** `cargo test -p pt-repo` (no database needed)

## Step 6: Update .env.example

**File:** `.env.example`
**Change:** Add Neon connection string examples with `sslnegotiation=direct` and `statement_cache_size=0` comments.

**Verify:** Visual inspection.

## Step 7: Claim milestone in progress.rs

**File:** `tests/scenarios/src/progress.rs`
**Change:** Add milestone for connection hardening, linking to T-020-02.

**Verify:** `just scenarios`

## Step 8: Quality gate

**Command:** `just check`
**Expected:** fmt-check, lint, test, scenarios all pass.

## Testing Strategy

| Test | Type | Database Required | What It Verifies |
|------|------|-------------------|------------------|
| `io_error_is_transient` | Unit | No | Error classification logic |
| `config_error_is_not_transient` | Unit | No | Error classification logic |
| `pool_connects_with_defaults` | Integration | Yes | Default config connects to local PG |
| `pool_connects_with_custom_config` | Integration | Yes | Custom config honored |
| `pool_rejects_invalid_url` | Integration | No | Config errors not retried |

Existing 19 CRUD integration tests serve as regression — they verify that pool changes don't break query execution.
