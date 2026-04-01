# T-020-01 Design: sqlx::test Migration

## Problem

19 integration tests in pt-repo are permanently `#[ignore]`'d. They can't run in CI, can't run in parallel, share database state, and require manual cleanup. This defeats the testing philosophy — these tests verify real customer value paths (CRUD for projects, zones, materials, tiers) but we have zero confidence they pass.

## Options Evaluated

### Option A: Feature-gated `#[sqlx::test]` with `integration` feature

- Add `integration` cargo feature to pt-repo
- Gate each test file with `#![cfg(feature = "integration")]`
- Replace `#[tokio::test]` + `#[ignore]` with `#[sqlx::test(migrations = "../../migrations")]`
- Remove common/mod.rs entirely
- `just test` = `cargo test --workspace` (feature not enabled, integration tests not compiled)
- `just test-integration` = `cargo test -p pt-repo --features integration`

**Pros:**
- Tests don't compile at all when feature is off — zero overhead in `just test`
- Clean separation: no database dependency in default workflow
- Graceful "skip" = not compiled, not "skip at runtime with a warning"
- CI doesn't need Postgres service for basic checks

**Cons:**
- Feature flags add a dimension to the build matrix
- `cargo test --workspace --features integration` is per-package, not workspace-wide (minor)

### Option B: Runtime DATABASE_URL check with custom skip macro

- Keep `#[tokio::test]`, remove `#[ignore]`
- Write a macro/helper that checks `DATABASE_URL` and returns `Ok(())` early if missing
- `just test` runs everything; tests without DB silently pass

**Pros:**
- No feature flags
- All tests always compiled (catches compile errors)

**Cons:**
- "Silently pass" is misleading — test appears green when it ran nothing
- Violates CLAUDE.md rule: "Do not write tests that pass trivially"
- Doesn't use sqlx::test, so no per-test database isolation
- Manual cleanup still needed

**Rejected.** Silent passes violate testing philosophy.

### Option C: `#[sqlx::test]` without feature gating, `--exclude` in justfile

- Use `#[sqlx::test]` directly (no feature flag)
- `just test` = `cargo test --workspace --exclude pt-repo`
- `just test-integration` = `cargo test -p pt-repo`

**Pros:**
- Simpler — no feature flag
- Tests always compiled

**Cons:**
- `--exclude pt-repo` excludes ALL pt-repo tests, including any future unit tests in src/
- Blunt instrument — can't run pt-repo unit tests in `just test`
- If someone runs `cargo test -p pt-repo` without DATABASE_URL, they get a confusing panic

**Rejected.** Too coarse — excludes unit tests too.

### Option D: Hybrid — feature gate + `#[sqlx::test]` + seed helpers

Same as Option A but keep common/mod.rs with seed-only helpers (create_tenant, create_project_with_zones) that take a `&PgPool` parameter. This avoids duplicating fixture creation across test files.

**Pros:**
- All of Option A's benefits
- DRY fixture creation
- Seed helpers are useful as tests grow

**Cons:**
- Slightly more code to maintain
- Current tests already have per-file `setup()` functions that work well

**Assessment:** The current per-file `setup()` pattern is fine for 19 tests. Adding shared seed helpers is premature. If test count grows significantly, a follow-up ticket can extract common fixtures. Keep it simple now.

## Decision: Option A — Feature-gated `#[sqlx::test]`

### Rationale

1. **Clean separation.** Feature flag means `just test` has zero database dependency — not "tests skip", but "tests don't exist in this build". This is the strongest guarantee.

2. **sqlx::test handles everything.** Per-test ephemeral database, automatic migration, automatic cleanup. Eliminates common/mod.rs entirely.

3. **Parallel-safe.** Each test gets its own database. No shared state, no ordering issues, no cleanup.

4. **Minimal changes.** Each test conversion is mechanical: change the attribute, change the function signature, remove common:: calls and cleanup code.

5. **CI unchanged.** `just test` works without Postgres. Integration tests can be added to CI later (T-020-02 or separate ticket with Postgres service).

### Test conversion pattern

Before:
```rust
#[tokio::test]
#[ignore = "Requires Postgres (S.INFRA.1), tracked in T-003-02"]
async fn test_name() {
    let pool = common::test_pool().await;
    common::setup_test_db(&pool).await;
    // ... test body ...
    // Cleanup
    thing::delete(&pool, id).await.unwrap();
}
```

After:
```rust
#[sqlx::test(migrations = "../../migrations")]
async fn test_name(pool: PgPool) {
    // ... test body (no cleanup needed) ...
}
```

### Justfile changes

```just
test:           # cargo test --workspace (no integration feature = no DB tests)
test-integration:  # DATABASE_URL=... cargo test -p pt-repo --features integration
```

### Feature flag design

In `crates/pt-repo/Cargo.toml`:
```toml
[features]
integration = []
```

In each test file:
```rust
#![cfg(feature = "integration")]
```

This is the idiomatic Rust pattern for conditional integration tests.
