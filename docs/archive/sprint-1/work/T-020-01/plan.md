# T-020-01 Plan: sqlx::test Migration

## Step 1: Add `migrate` feature to workspace sqlx dependency

**File:** `Cargo.toml` (workspace root, line 19)

Add `"migrate"` to the sqlx features list. This enables `#[sqlx::test]` and `sqlx::migrate!()`.

**Verify:** `cargo check --workspace` compiles cleanly.

## Step 2: Add `integration` feature and sqlx dev-dep to pt-repo

**File:** `crates/pt-repo/Cargo.toml`

- Add `[features]` section with `integration = []`
- Add `sqlx = { workspace = true }` to `[dev-dependencies]`

**Verify:** `cargo check -p pt-repo` compiles. `cargo check -p pt-repo --features integration` also compiles.

## Step 3: Delete common/mod.rs

**File:** `crates/pt-repo/tests/common/mod.rs` — delete entirely

This will temporarily break test compilation (tests reference `mod common`), which is fine since we convert them in the next steps.

## Step 4: Convert material_test.rs (5 tests)

**File:** `crates/pt-repo/tests/material_test.rs`

For each of the 5 tests:
1. Add `#![cfg(feature = "integration")]` at file top
2. Remove `mod common;`
3. Remove `setup()` function — inline `tenant::create()` into each test
4. Replace `#[tokio::test]` + `#[ignore]` with `#[sqlx::test(migrations = "../../migrations")]`
5. Change signature to `async fn name(pool: PgPool)`
6. Remove cleanup (delete) calls at end of each test
7. Keep `sample_material()` helper unchanged

**Verify:** `cargo test -p pt-repo --features integration --no-run` compiles.

## Step 5: Convert project_test.rs (5 tests)

**File:** `crates/pt-repo/tests/project_test.rs`

Same conversion pattern as Step 4. Inline tenant creation. Remove cleanup.

**Verify:** Compiles with `--no-run`.

## Step 6: Convert zone_test.rs (4 tests)

**File:** `crates/pt-repo/tests/zone_test.rs`

Same pattern. Inline tenant + project creation. Remove cleanup.

**Verify:** Compiles with `--no-run`.

## Step 7: Convert tier_test.rs (4 tests)

**File:** `crates/pt-repo/tests/tier_test.rs`

Remove `TestFixture` struct and `setup()`. Inline full fixture chain (tenant -> project -> zone -> material) into each test. Remove cleanup.

**Verify:** Compiles with `--no-run`.

## Step 8: Convert round_trip_test.rs (1 test)

**File:** `crates/pt-repo/tests/round_trip_test.rs`

Remove `mod common;` and inline calls. Remove final cleanup. This is the largest single test.

**Verify:** Compiles with `--no-run`.

## Step 9: Update justfile

Add `test-integration` recipe with default DATABASE_URL matching docker-compose. Update `check` recipe comment.

**Verify:** `just --list` shows new recipe.

## Step 10: Full verification

1. `cargo test --workspace` — passes (integration tests not compiled)
2. `just test` — passes without database
3. `just lint` — no clippy warnings
4. `just fmt-check` — formatted
5. If database available: `just test-integration` — all 19 tests pass
6. `just scenarios` — no regression

## Testing Strategy

- **Unit tests (existing):** Unaffected. `cargo test --workspace` still runs all non-integration tests.
- **Integration tests (19 converted):** Only run with `--features integration` + DATABASE_URL. Each gets its own ephemeral database via sqlx::test.
- **Verification without database:** `cargo test -p pt-repo` should produce 0 tests (all gated behind feature). This is correct — no false passes.
- **Verification with database:** `just dev-db && just test-integration` should run and pass all 19 tests in < 30 seconds.
