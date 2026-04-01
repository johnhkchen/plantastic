# T-020-01 Progress: sqlx::test Migration

## Completed

### Step 1: Add `migrate` feature to workspace sqlx ✅
- Added `"migrate"` to `Cargo.toml` workspace sqlx features

### Step 2: Add `integration` feature and sqlx dev-dep to pt-repo ✅
- Added `[features] integration = []` to `crates/pt-repo/Cargo.toml`
- Added `sqlx = { workspace = true }` to dev-dependencies

### Step 3: Delete common/mod.rs ✅
- Removed `crates/pt-repo/tests/common/mod.rs` and `common/` directory

### Step 4: Convert material_test.rs (5 tests) ✅
- Added `#![cfg(feature = "integration")]`
- Replaced `#[tokio::test]` + `#[ignore]` with `#[sqlx::test(migrations = "../../migrations")]`
- Changed signatures to `async fn name(pool: PgPool)`
- Removed `mod common;`, `setup()`, and cleanup calls

### Step 5: Convert project_test.rs (5 tests) ✅
- Same conversion pattern as Step 4

### Step 6: Convert zone_test.rs (4 tests) ✅
- Same conversion. Added `create_project()` helper (takes &PgPool, no DB setup)

### Step 7: Convert tier_test.rs (4 tests) ✅
- Replaced `TestFixture` struct with `create_fixture()` helper taking &PgPool
- Same conversion pattern

### Step 8: Convert round_trip_test.rs (1 test) ✅
- Same conversion. Removed final cascade cleanup

### Step 9: Update justfile ✅
- Added `test-integration` recipe with default DATABASE_URL
- Added NOTE comment to `check` recipe

### Step 10: Verification ✅
- `cargo test --workspace --no-run` — compiles clean
- `cargo test -p pt-repo --features integration --no-run` — compiles clean
- `just fmt` — formatted (minor reformats applied)
- `just lint` — zero warnings
- `just test` — passes (0 ignored repo tests, feature not enabled)
- `just scenarios` — 58.0 min / 240.0 min (24.2%), no regression

## Deviations from Plan

None. All steps executed as planned.

## Remaining

- Integration test execution with real Postgres not verified in this session (requires `just dev-db` running)
- Performance measurement (< 30s target) deferred to when database is available
