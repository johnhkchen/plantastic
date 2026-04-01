# T-020-02 Design: Connection Hardening

## Decision 1: API Surface — Config Struct vs Defaults-Only

### Option A: `PoolConfig` struct with `Default` impl
```rust
pub struct PoolConfig {
    pub connect_timeout: Duration,
    pub acquire_timeout: Duration,
    pub max_connections: u32,
    pub min_connections: u32,
    pub max_retries: u32,
    pub initial_backoff: Duration,
}
pub async fn create_pool(database_url: &str, config: &PoolConfig) -> Result<PgPool, RepoError>
```
Pros: Fully configurable, testable with different configs, caller controls behavior.
Cons: Breaking change to `create_pool()` signature; plantastic-api must update.

### Option B: Keep `create_pool(&str)` signature, add `create_pool_with_config()`
```rust
pub async fn create_pool(database_url: &str) -> Result<PgPool, RepoError>  // defaults
pub async fn create_pool_with_config(database_url: &str, config: &PoolConfig) -> Result<PgPool, RepoError>
```
Pros: Backwards-compatible; existing call sites unchanged.
Cons: Two functions doing the same thing; `create_pool` becomes a thin wrapper.

### Option C: `PoolConfig` with `Default`, update the one call site
Same as Option A but accept the minor churn at the API call site.

**Decision: Option C.** There is exactly one call site (`main.rs:24`). The breaking change is trivial — pass `&PoolConfig::default()`. A config struct with `Default` is the idiomatic Rust pattern for this. Option B adds unnecessary API surface. The config struct also makes testing easy (short timeouts, zero retries for fast tests).

## Decision 2: Retry Scope — Pool Creation vs Individual Connections

### Option A: Retry the entire `PgPoolOptions::connect()` call
Retry creates a new pool on each attempt. On success, return the pool.

### Option B: Use `PgPoolOptions::after_connect()` callback with retry inside
Hook into individual connection establishment within the pool.

**Decision: Option A.** The ticket specifies "connection establishment retries on transient failures" — this means retrying pool creation, not individual query connections. The pool's internal connection management handles ongoing reconnection after the pool is established. The cold-start problem is specifically at initial pool creation time (Lambda cold-start + Neon cold-start simultaneously). Retrying pool creation is simpler and directly addresses the problem.

## Decision 3: `connect_timeout` Mechanism

sqlx's `PgConnectOptions::connect_timeout()` sets the TCP-level timeout per connection attempt. Two ways to apply it:

### Option A: Parse URL into `PgConnectOptions`, set timeout, pass to pool
```rust
let opts = PgConnectOptions::from_str(database_url)?
    .connect_timeout(config.connect_timeout);
PgPoolOptions::new().connect_with(opts).await
```

### Option B: Append `connect_timeout` as a query parameter to the URL
Modify the URL string before passing to `.connect()`.

**Decision: Option A.** Explicit and type-safe. URL manipulation is fragile (query param encoding, existing params). `PgConnectOptions::from_str()` parses the URL and preserves all existing params (`sslnegotiation=direct`, `sslmode`, etc.), then we overlay our timeout.

## Decision 4: Transient Error Detection

Need a function `is_transient(err: &sqlx::Error) -> bool` to decide whether to retry.

**Design:**
- `Io(_)` → transient (TCP failures, DNS, timeouts)
- `Tls(_)` → transient (handshake can fail on cold-start, especially with Neon)
- `PoolTimedOut` → transient (pool couldn't establish connections in time)
- Everything else → permanent (configuration errors, auth failures, etc.)

This is conservative — we retry on network-level failures only. Auth failures (`Database` variant with specific PG codes) are not retried because they won't self-heal.

## Decision 5: Backoff Strategy

Ticket specifies: 3 attempts, exponential backoff starting at 500ms.

Attempt 1: immediate
Attempt 2: sleep 500ms, then try
Attempt 3: sleep 1000ms, then try

Total worst-case wall time: ~1.5s of backoff + 3 × connect_timeout. With 15s connect_timeout, worst case is ~46.5s. This fits within Lambda's 30s default timeout only if connect_timeout is reduced — but the ticket says default 15s. The caller (Lambda config) should set a longer function timeout (60s) or use shorter connect_timeout via PoolConfig.

No jitter needed — there's only one Lambda instance trying to connect at a time.

## Decision 6: Logging

Add `tracing` dependency to pt-repo. Log at:
- `WARN` on each retry: attempt number, error, backoff duration
- `INFO` on successful connection: total attempts, total elapsed time
- `DEBUG` on pool configuration details at creation time

## Decision 7: `sslnegotiation=direct` and PgBouncer Support

**No code changes needed.** sqlx 0.8 parses these from the connection URL:
- `?sslnegotiation=direct` → honored by the driver
- `-pooler` hostname → just a DNS name, no special handling
- `?statement_cache_size=0` → user adds to URL for PgBouncer transaction mode

Document these in the `PoolConfig` and `create_pool_with_config` docstrings. Add them to `.env.example` comments.

## Rejected Alternatives

- **External retry crate (backoff, retry):** Overkill for 3-attempt exponential backoff. 15 lines of code.
- **Connection health check callback:** sqlx pool already does health checks internally. Adding `test_before_acquire` would slow every connection acquisition.
- **Custom connection wrapper type:** No value — `PgPool` is the correct abstraction. Wrapping it hides sqlx's API.

## Summary

1. Add `PoolConfig` struct with `Default` impl (Lambda-tuned defaults)
2. Add `create_pool_with_config()` as the main implementation
3. Refactor `create_pool()` to delegate with `PoolConfig::default()`
4. Parse URL → `PgConnectOptions` → apply `connect_timeout` → `PgPoolOptions::connect_with()`
5. Wrap connect in retry loop: 3 attempts, 500ms exponential backoff, transient-only
6. Log retries at WARN, success at INFO
7. Add `tracing` to pt-repo dependencies
8. Update API call site to use `PoolConfig::default()`
9. Document Neon connection string patterns
