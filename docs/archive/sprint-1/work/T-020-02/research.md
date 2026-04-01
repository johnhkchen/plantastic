# T-020-02 Research: Connection Hardening

## Current Pool Implementation

**File:** `crates/pt-repo/src/pool.rs` (24 lines)

Single function `create_pool(database_url: &str) -> Result<PgPool, RepoError>` with:
- `max_connections=5`, `min_connections=0`, `idle_timeout=30s`, `acquire_timeout=3s`
- No `connect_timeout` — defaults to sqlx's internal default (undefined/infinite)
- No retry logic — single attempt, fail or succeed
- No logging of connection attempts
- Connection string passed directly to `PgPoolOptions::connect()`

**Call site:** `crates/plantastic-api/src/main.rs:24` — calls `pt_repo::create_pool()` at startup, exits on failure. No retry wrapper.

## sqlx 0.8 Connection Model

`PgPoolOptions` exposes:
- `.max_connections(u32)` / `.min_connections(u32)` — pool sizing
- `.idle_timeout(Duration)` — drop idle connections after this
- `.acquire_timeout(Duration)` — how long to wait for a connection from the pool
- `.connect(url)` — parses URL into `PgConnectOptions`, then creates pool

`PgConnectOptions` (from URL parsing or builder) exposes:
- `.connect_timeout(Duration)` — TCP-level connection timeout per attempt

Connection string query parameters honored by sqlx 0.8 postgres driver:
- `sslmode=require|prefer|disable|verify-ca|verify-full`
- `sslnegotiation=direct` — skips StartupMessage/SSLRequest handshake, goes straight to TLS
- `statement_cache_size=N` — PgBouncer transaction mode requires `statement_cache_size=0`
- `options=-c search_path=...` — session-level settings

**Key finding:** `sslnegotiation=direct` and `statement_cache_size` are parsed from the URL automatically by sqlx. No special code needed — the user just includes them in `DATABASE_URL`.

## Neon Serverless Behavior

Neon cold-starts compute nodes on first connection. Documented latency:
- **Pooled (`-pooler` suffix):** PgBouncer in transaction mode, compute may or may not be cold. Connection through PgBouncer is typically 100-500ms. Cold compute behind PgBouncer adds 1-5s.
- **Direct (no `-pooler`):** TCP directly to compute node. Cold start can take 3-8s. Warm is ~50ms.
- **`sslnegotiation=direct`:** Saves one round-trip (no SSLRequest/response). ~50-100ms improvement on remote connections.

**PgBouncer transaction mode implications:**
- No prepared statements across transactions — sqlx 0.8 sends `statement_cache_size=0` when detected, or user sets it in URL
- No `LISTEN/NOTIFY`
- No session-level `SET` commands persisting across queries
- pt-repo uses simple parameterized queries (no prepared statements cache dependency) — compatible

## Error Classification for Retry

`sqlx::Error` variants relevant to connection:
- `Io(std::io::Error)` — TCP connection failures, timeouts, DNS resolution
- `Tls(...)` — TLS handshake failures
- `Configuration(...)` — bad URL, invalid options (NOT retriable)
- `Database(DatabaseError)` — server-side errors (NOT retriable for connection)
- `PoolTimedOut` — acquire_timeout exceeded (retriable at pool level, but indicates pool exhaustion)
- `PoolClosed` — pool shut down (NOT retriable)
- `WorkerCrashed` — internal sqlx error (NOT retriable)

Retriable for pool creation: `Io`, `Tls` (transient handshake), `PoolTimedOut`.
Not retriable: `Configuration`, `Database`, `PoolClosed`, `WorkerCrashed`.

## Existing Dependencies

pt-repo's Cargo.toml dependencies relevant to this work:
- `sqlx` (workspace: 0.8, features: runtime-tokio, tls-rustls, postgres, migrate)
- `tokio` (workspace: macros, rt-multi-thread) — needed for `tokio::time::sleep`
- `thiserror` (workspace: 2) — available but not currently used in error.rs (manual impl)
- **Missing:** `tracing` — needed for WARN-level retry logging. Currently only in plantastic-api.

## Integration Test Infrastructure

19 tests use `#[sqlx::test(migrations = "../../migrations")]` which creates its own pool per test. These tests will NOT exercise the custom `create_pool()` — they use sqlx's built-in test pool. This is correct: integration tests verify CRUD behavior, not pool configuration.

Pool-specific tests (timeout, retry) need to be unit tests or dedicated integration tests that call `create_pool()` directly against local Docker Postgres.

## API Call Site Analysis

`plantastic-api/src/main.rs:24` currently does:
```rust
let pool = pt_repo::create_pool(&database_url).await.unwrap_or_else(|e| { ... exit(1) });
```

With retry logic inside `create_pool()`, this call site needs no changes — retries are transparent. The API already logs at error level on failure. Retry WARN logs will appear before either success or final failure.

## Workspace tracing Setup

`tracing = "0.1"` is a workspace dependency. pt-repo does not currently depend on it. Adding `tracing.workspace = true` to pt-repo's Cargo.toml is straightforward.

## Summary of Constraints

1. `create_pool()` is the only public pool API — changing its signature is a breaking change for plantastic-api
2. sqlx handles `sslnegotiation=direct` and PgBouncer params from the URL automatically
3. Retry logic belongs inside `create_pool()` wrapping the `.connect()` call
4. `tracing` dependency needs to be added to pt-repo
5. `connect_timeout` needs `PgConnectOptions` parsing from the URL, then applying timeout before connecting
6. Existing integration tests are unaffected — they use sqlx's test infrastructure, not `create_pool()`
