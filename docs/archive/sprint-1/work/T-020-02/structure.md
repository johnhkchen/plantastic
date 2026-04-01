# T-020-02 Structure: Connection Hardening

## Files Modified

### 1. `crates/pt-repo/Cargo.toml`
Add `tracing` workspace dependency.

### 2. `crates/pt-repo/src/pool.rs` (rewrite)
Current: 24 lines, single `create_pool()` function.
After: ~120 lines.

**New types:**
```rust
pub struct PoolConfig {
    pub connect_timeout: Duration,    // default: 15s
    pub acquire_timeout: Duration,    // default: 10s
    pub idle_timeout: Duration,       // default: 30s
    pub max_connections: u32,         // default: 5
    pub min_connections: u32,         // default: 0
    pub max_retries: u32,            // default: 3
    pub initial_backoff: Duration,   // default: 500ms
}

impl Default for PoolConfig { ... }  // Lambda-tuned defaults per ticket AC
```

**Modified functions:**
```rust
// Delegates to create_pool_with_config with defaults
pub async fn create_pool(database_url: &str) -> Result<PgPool, RepoError>

// Main implementation
pub async fn create_pool_with_config(database_url: &str, config: &PoolConfig) -> Result<PgPool, RepoError>
```

**New private functions:**
```rust
// Classifies sqlx errors as transient (retriable) or permanent
fn is_transient(err: &sqlx::Error) -> bool

// Retry loop: attempts connect up to max_retries times with exponential backoff
async fn connect_with_retry(pool_opts: PgPoolOptions, connect_opts: PgConnectOptions, config: &PoolConfig) -> Result<PgPool, sqlx::Error>
```

### 3. `crates/pt-repo/src/lib.rs`
Add re-export: `pub use pool::PoolConfig;`
Keep existing `pub use pool::create_pool;`

### 4. `crates/plantastic-api/src/main.rs`
No change needed — `create_pool(&str)` signature preserved.

### 5. `.env.example`
Add documented Neon connection string examples:
- Neon direct with `sslnegotiation=direct`
- Neon pooled with `statement_cache_size=0`

## Files NOT Modified

- `crates/pt-repo/src/error.rs` — `RepoError::Database(sqlx::Error)` already handles all connection errors. No new variants needed; retry logic is internal to pool.rs.
- `crates/pt-repo/tests/*.rs` — existing 19 integration tests use `#[sqlx::test]`, not `create_pool()`. They verify CRUD, not pool behavior.
- `crates/pt-repo/src/{tenant,project,zone,material,tier_assignment,convert}.rs` — no changes.

## New Files

### 6. `crates/pt-repo/tests/pool_test.rs`
Feature-gated integration test (`#[cfg(feature = "integration")]`) that:
- Calls `create_pool()` against local Docker Postgres
- Verifies pool connects on first attempt (no retries for local)
- Calls `create_pool_with_config()` with custom config
- Verifies `is_transient` classification via unit-testable scenarios

## Module Boundaries

```
pt-repo (lib.rs)
├── pub use pool::{create_pool, PoolConfig}   // public API
├── pub use error::RepoError                   // public API
├── pool.rs
│   ├── pub struct PoolConfig + Default        // config type
│   ├── pub fn create_pool()                   // convenience (delegates)
│   ├── pub fn create_pool_with_config()       // main entry point
│   ├── fn is_transient()                      // private error classifier
│   └── fn connect_with_retry()                // private retry loop
├── error.rs                                   // unchanged
└── {crud modules}                             // unchanged
```

## Dependency Graph Change

```
pt-repo
  ├── sqlx (existing)
  ├── tokio (existing — needed for tokio::time::sleep in retry backoff)
  ├── tracing (NEW — for WARN/INFO/DEBUG logging)
  └── ... (unchanged)
```

## Ordering

1. Add `tracing` dependency to Cargo.toml
2. Rewrite pool.rs (PoolConfig, is_transient, connect_with_retry, create_pool_with_config)
3. Update lib.rs exports
4. Add pool_test.rs
5. Update .env.example with Neon patterns
6. Run `just check` to verify
