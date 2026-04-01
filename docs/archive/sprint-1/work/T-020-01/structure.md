# T-020-01 Structure: sqlx::test Migration

## File Changes

### Modified: `Cargo.toml` (workspace root)

Add `migrate` feature to workspace sqlx dependency:
```toml
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "postgres", "chrono", "uuid", "json", "rust_decimal", "migrate"] }
```

The `migrate` feature enables `sqlx::migrate!()` macro and `#[sqlx::test]` attribute.

### Modified: `crates/pt-repo/Cargo.toml`

Add feature flag and sqlx dev-dependency override:
```toml
[features]
integration = []

[dev-dependencies]
sqlx = { workspace = true }   # needed for #[sqlx::test] macro
pt-test-utils = { path = "../pt-test-utils" }
```

sqlx must be in dev-dependencies because test files use `#[sqlx::test]` attribute directly. The workspace dep brings in all needed features including `migrate`.

### Deleted: `crates/pt-repo/tests/common/mod.rs`

Entire file removed. `#[sqlx::test]` replaces both `test_pool()` and `setup_test_db()`.

### Modified: `crates/pt-repo/tests/material_test.rs`

- Add `#![cfg(feature = "integration")]` at top
- Remove `mod common;`
- Remove local `setup()` function (inline tenant creation into each test)
- Convert 5 tests: `#[tokio::test]` + `#[ignore]` -> `#[sqlx::test(migrations = "../../migrations")]`
- Each test: `async fn name(pool: PgPool)` instead of `async fn name()`
- Remove all cleanup (`material::delete`, `project::delete`) calls
- Keep `sample_material()` helper (it doesn't touch the DB)

### Modified: `crates/pt-repo/tests/project_test.rs`

- Add `#![cfg(feature = "integration")]` at top
- Remove `mod common;`
- Remove local `setup()` function
- Convert 5 tests to `#[sqlx::test]`
- Inline tenant creation: `pt_repo::tenant::create(&pool, "Test Co")`
- Remove cleanup calls

### Modified: `crates/pt-repo/tests/zone_test.rs`

- Add `#![cfg(feature = "integration")]` at top
- Remove `mod common;`
- Remove local `setup()` function
- Convert 4 tests to `#[sqlx::test]`
- Inline tenant + project creation at start of each test
- Remove cleanup calls

### Modified: `crates/pt-repo/tests/tier_test.rs`

- Add `#![cfg(feature = "integration")]` at top
- Remove `mod common;`
- Remove `TestFixture` struct (pool comes from parameter, rest inlined)
- Remove `setup()` function
- Convert 4 tests to `#[sqlx::test]`
- Inline full fixture creation (tenant -> project -> zone -> material)
- Remove cleanup calls

### Modified: `crates/pt-repo/tests/round_trip_test.rs`

- Add `#![cfg(feature = "integration")]` at top
- Remove `mod common;`
- Convert 1 test to `#[sqlx::test]`
- Remove cleanup at end (cascade delete)

### Modified: `justfile`

Add `test-integration` recipe after existing test recipes:

```just
# Run integration tests (requires DATABASE_URL — start with `just dev-db`)
test-integration:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Running integration tests (requires Postgres)..."
    DATABASE_URL="${DATABASE_URL:-postgres://plantastic:plantastic@localhost:5432/plantastic}" \
        cargo test -p pt-repo --features integration 2>&1
    echo ""
    echo "Integration tests passed."
```

Default DATABASE_URL matches docker-compose config so `just dev-db && just test-integration` works with zero config.

Update `check` recipe comment to document full validation:

```just
# Run all quality gates: format, lint, test, scenarios
# NOTE: Full validation also requires `just dev-db && just test-integration`
check: fmt-check lint test scenarios
```

### NOT Modified

- `migrations/` — no changes
- `docker-compose.yml` — no changes
- `.github/workflows/ci.yml` — no changes (unit tests still work, integration tests added to CI in future ticket)
- `crates/pt-repo/src/` — no production code changes
- `crates/pt-test-utils/` — no changes (not used by integration tests)

## Module Boundaries

- `#[sqlx::test]` is a proc-macro from sqlx — no new internal abstractions needed
- Integration tests remain in `crates/pt-repo/tests/` (standard Rust test location)
- Feature flag `integration` is local to pt-repo crate only
- No cross-crate interface changes

## Ordering

1. Cargo.toml changes first (features must exist before test files reference them)
2. Delete common/mod.rs
3. Convert test files (order doesn't matter — they're independent)
4. Update justfile last (tests must compile before recipe is useful)
