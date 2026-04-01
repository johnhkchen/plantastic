# T-020-01 Review: sqlx::test Migration

## Summary

Migrated all 19 pt-repo integration tests from manual `#[tokio::test]` + `#[ignore]` pattern to `#[sqlx::test]` with feature-gated compilation. Each test now gets an ephemeral database with migrations auto-applied, eliminating shared state, manual cleanup, and permanently-ignored tests.

## Files Changed

### Modified
| File | Change |
|------|--------|
| `Cargo.toml` | Added `"migrate"` to workspace sqlx features |
| `crates/pt-repo/Cargo.toml` | Added `[features] integration = []` and `sqlx` dev-dep |
| `crates/pt-repo/tests/material_test.rs` | Converted 5 tests to `#[sqlx::test]` |
| `crates/pt-repo/tests/project_test.rs` | Converted 5 tests to `#[sqlx::test]` |
| `crates/pt-repo/tests/zone_test.rs` | Converted 4 tests to `#[sqlx::test]` |
| `crates/pt-repo/tests/tier_test.rs` | Converted 4 tests to `#[sqlx::test]` |
| `crates/pt-repo/tests/round_trip_test.rs` | Converted 1 test to `#[sqlx::test]` |
| `justfile` | Added `test-integration` recipe, updated `check` comment |

### Deleted
| File | Reason |
|------|--------|
| `crates/pt-repo/tests/common/mod.rs` | Replaced entirely by sqlx::test infrastructure |

### Not Modified
- `migrations/` — unchanged
- `docker-compose.yml` — unchanged
- `.github/workflows/ci.yml` — unchanged (unit tests don't need DB)
- `crates/pt-repo/src/` — no production code changes

## Acceptance Criteria Status

### sqlx::test migration ✅
- [x] All tests use `#[sqlx::test(migrations = "../../migrations")]`
- [x] Each test receives a `PgPool` parameter
- [x] Removed `common/mod.rs` entirely (no setup/teardown helpers)
- [x] Removed all 19 `#[ignore]` annotations
- [x] Tests fully isolated (each gets own ephemeral DB)

### Test gating ✅
- [x] `just test` runs only unit tests (integration feature not enabled = tests not compiled)
- [x] `just test-integration` runs repo integration tests with DATABASE_URL
- [x] Integration tests skip gracefully — they simply don't exist in the default build
- [x] `check` recipe documents that full validation requires `just dev-db`

### Verification ⚠️ (partially verified)
- [x] `just test` passes without database — confirmed
- [x] `just lint` passes — confirmed
- [x] `just fmt-check` passes — confirmed
- [x] `just scenarios` — 58.0/240.0 min, no regression
- [ ] `just dev-db && just test-integration` — not run (requires active Docker)
- [ ] Test suite < 30 seconds — not measured yet

## Scenario Dashboard

Before: 58.0 min / 240.0 min (24.2%), 8 pass, 0 fail
After:  58.0 min / 240.0 min (24.2%), 8 pass, 0 fail

No regression. This is infrastructure work — it doesn't flip scenarios but unblocks S.INFRA.1/S.INFRA.2 by making repo tests actually runnable.

## Test Coverage

- **19 integration tests** converted (material: 5, project: 5, zone: 4, tier: 4, round-trip: 1)
- **0 tests removed** — all test logic preserved
- **0 new tests added** — scope was migration, not expansion
- All cleanup code removed (sqlx::test drops ephemeral DB automatically)
- Test assertions unchanged — same verification logic

## Design Decisions

1. **Feature flag over `--exclude`**: Chose `cfg(feature = "integration")` because `--exclude pt-repo` would also skip any future unit tests in pt-repo's `src/`.

2. **No shared seed helpers**: Kept per-file setup functions (`create_project()`, `create_fixture()`) rather than extracting a shared module. 19 tests don't justify the abstraction. The old `common/mod.rs` was only needed for pool/migration management which sqlx::test now handles.

3. **Default DATABASE_URL in justfile**: `test-integration` defaults to the docker-compose credentials so `just dev-db && just test-integration` works zero-config.

## Open Concerns

1. **Integration tests not yet run against real Postgres.** The tests compile and the pattern is correct per sqlx 0.8 docs, but actual execution needs Docker running. First person to run `just dev-db && just test-integration` should verify all 19 pass and measure timing.

2. **CI doesn't run integration tests.** This is intentional — CI has no Postgres service. A future ticket could add a Postgres service to the GitHub Actions workflow, or integration tests could run in a separate CI job with Neon branching (T-021-02).

3. **PostGIS in ephemeral databases.** `#[sqlx::test]` creates a new database per test and runs migrations. Migration 001 does `CREATE EXTENSION IF NOT EXISTS postgis`. This should work because `postgis/postgis:18-3.6` installs the extension at the server level, making it available in any database. If this fails, the fix is to ensure the PostGIS shared library is installed (it is, in our Docker image).
