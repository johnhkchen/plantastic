# T-020-02 Progress: Connection Hardening

## Completed

1. **Added tracing dependency** — `tracing.workspace = true` in pt-repo Cargo.toml
2. **Rewrote pool.rs** — PoolConfig struct, is_transient(), connect_with_retry(), create_pool_with_config()
3. **Updated lib.rs exports** — pub use pool::{create_pool, create_pool_with_config, PoolConfig}
4. **Added unit tests** — 6 tests in pool::tests (config defaults, error classification)
5. **Added integration tests** — 4 tests in pool_test.rs (connect, custom config, invalid URL, max_connections)
6. **Updated .env.example** — Neon direct and pooled connection string examples
7. **Claimed milestone** — in tests/scenarios/src/progress.rs

## Deviations from Plan

- `PgConnectOptions` in sqlx 0.8 does not expose `connect_timeout()` method. Used `tokio::time::timeout` wrapping the `connect_with()` call instead. This is actually better — it covers the entire connection establishment (TCP + TLS + auth), not just TCP.
- API call site (`main.rs`) unchanged — `create_pool(&str)` signature preserved with delegation to `create_pool_with_config()`.

## Remaining

- Run `just check` quality gate
- Write review.md
