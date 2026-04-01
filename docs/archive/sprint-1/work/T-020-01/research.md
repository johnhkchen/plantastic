# T-020-01 Research: sqlx::test Migration

## Current Test Infrastructure

### Test Files (19 tests total, all `#[ignore]`'d)

| File | Tests | Setup Pattern |
|------|-------|---------------|
| `crates/pt-repo/tests/material_test.rs` | 5 | local `setup()` -> `(PgPool, Uuid)` |
| `crates/pt-repo/tests/project_test.rs` | 5 | local `setup()` -> `(PgPool, Uuid)` |
| `crates/pt-repo/tests/zone_test.rs` | 4 | local `setup()` -> `(PgPool, Uuid, Uuid)` |
| `crates/pt-repo/tests/tier_test.rs` | 4 | local `setup()` -> `TestFixture` struct |
| `crates/pt-repo/tests/round_trip_test.rs` | 1 | inline `common::test_pool()` + `setup_test_db()` |

All tests follow the same pattern:
1. `#[tokio::test]` + `#[ignore = "Requires Postgres (S.INFRA.1), tracked in T-003-02"]`
2. Call `common::test_pool()` to create a `PgPool` from `DATABASE_URL`
3. Call `common::setup_test_db()` to apply migrations manually
4. Create tenant/project fixtures
5. Run assertions
6. Manual cleanup (delete operations)

### common/mod.rs (41 lines)

- `test_pool()`: Reads `DATABASE_URL`, calls `pt_repo::create_pool()`
- `setup_test_db()`: Reads migration files from `../../migrations/`, filters out `.down.sql`, sorts, executes raw SQL

Problems:
- All tests share the same database — no isolation
- Manual cleanup required and fragile
- Migration application is hand-rolled (not using sqlx's migrator)
- Tests can't run in parallel safely
- DATABASE_URL absence causes panic, not graceful skip

### sqlx Configuration

**Workspace Cargo.toml** (`sqlx` features):
```
runtime-tokio, tls-rustls, postgres, chrono, uuid, json, rust_decimal
```

Missing: `migrate` feature (needed for `#[sqlx::test]` and `sqlx::migrate!` macro)

**sqlx version**: 0.8 (supports `#[sqlx::test]`)

**pt-repo/Cargo.toml dev-dependencies**: Only `pt-test-utils`

### Migrations (6 up-migrations in `migrations/`)

| File | Creates |
|------|---------|
| 001-create-tenants.sql | PostGIS extension + `tenants` table |
| 002-create-projects.sql | `projects` + `project_status` enum |
| 003-create-zones.sql | `zones` with `GEOMETRY(POLYGON, 4326)` |
| 004-create-materials.sql | `materials` with JSONB `extrusion` |
| 005-create-tier-assignments.sql | `tier_assignments` linking materials to zones |
| 006-create-plants.sql | `plants` (platform-level, no tenant FK) |

Each has a corresponding `.down.sql`. Docker init script applies them in order.

### justfile Test Recipes

- `test`: `cargo test --workspace` with 120s timeout — runs everything including pt-repo (which currently all `#[ignore]`)
- `test-crate <crate>`: single-crate test
- `test-verbose`: `--nocapture`
- No `test-integration` recipe exists
- `check`: fmt-check -> lint -> test -> scenarios

### CI (`.github/workflows/ci.yml`)

- Runs `just test` without a database service
- Works today only because all repo tests are `#[ignore]`'d
- No Postgres service in CI workflow

### Docker Compose (from T-019-01)

- `postgis/postgis:18-3.6` on port 5432
- User/pass/db: `plantastic/plantastic/plantastic`
- Health check: `pg_isready`
- Migrations auto-applied via `docker-init-db.sh`

### How sqlx::test Works (sqlx 0.8)

1. Reads `DATABASE_URL` environment variable
2. Creates a uniquely-named temporary database per test
3. Applies migrations via `sqlx::migrate!()` macro
4. Passes a `PgPool` connected to the temp DB as the test function argument
5. Drops the temp database after test completes
6. Tests are fully isolated — safe to run in parallel
7. Requires: `migrate` feature + migration files at compile time

### Key Constraints

- PostGIS extension (`CREATE EXTENSION IF NOT EXISTS postgis`) is in migration 001 — sqlx::test must create databases with PostGIS available on the server
- The `postgis/postgis:18-3.6` Docker image ships with PostGIS extension — `CREATE EXTENSION` will work in any database on the server
- Migration path from test files: `../../migrations` (two levels up from `crates/pt-repo/`)
- `just test` must work without a database for CI and quick local iteration
- `just test-integration` needs to be a new recipe

### Files That Will Change

1. `Cargo.toml` (workspace) — add `migrate` feature to sqlx
2. `crates/pt-repo/Cargo.toml` — add `integration` feature flag
3. `crates/pt-repo/tests/material_test.rs` — convert 5 tests
4. `crates/pt-repo/tests/project_test.rs` — convert 5 tests
5. `crates/pt-repo/tests/zone_test.rs` — convert 4 tests
6. `crates/pt-repo/tests/tier_test.rs` — convert 4 tests
7. `crates/pt-repo/tests/round_trip_test.rs` — convert 1 test
8. `crates/pt-repo/tests/common/mod.rs` — delete (or replace with seed-only helpers)
9. `justfile` — add `test-integration` recipe, modify `test` to exclude repo integration tests
10. `.github/workflows/ci.yml` — no change needed (unit tests still work without DB)
