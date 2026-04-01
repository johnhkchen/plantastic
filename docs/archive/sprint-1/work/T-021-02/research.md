# Research: T-021-02 CI Neon Branching

## Objective

Add ephemeral Neon database branches to CI so integration tests run against real
Postgres+PostGIS in GitHub Actions, without Docker.

---

## Current CI Setup

**File:** `.github/workflows/ci.yml`

The Rust job runs four steps: `just fmt-check`, `just lint`, `just test`, `just scenarios`.
No database is available. Integration tests are skipped:

- **plantastic-api tests** (`crates/plantastic-api/tests/crud_test.rs`, `scan_test.rs`):
  Use `#[ignore = "Requires Postgres ..."]`. 10 tests in crud_test.rs, 6 in scan_test.rs.
  All read `DATABASE_URL` from env via `common::test_pool()`.

- **pt-repo tests** (`crates/pt-repo/tests/*.rs`):
  Gated by `#[cfg(feature = "integration")]`. 6 test files including `round_trip_test.rs`.
  Use `#[sqlx::test(migrations = "../../migrations")]` which creates an isolated database
  per test via sqlx's built-in test support.

- **Current budget:** ~5-7 minutes. 15-minute timeout. Plenty of headroom.

**File:** `justfile`

- `just test` — `cargo test --workspace` (skips `#[ignore]` tests, doesn't enable `integration` feature)
- `just test-integration` — `cargo test -p pt-repo --features integration` (manual, requires Postgres)

---

## Integration Test Patterns

### plantastic-api (16 ignored tests)

Tests in `crates/plantastic-api/tests/` follow this pattern:
1. `common::test_pool()` reads `DATABASE_URL` env var (panics if unset)
2. `common::setup_test_db(&pool)` runs all 6 migration .sql files
3. Tests create test data, exercise API routes via `tower::ServiceExt::oneshot`
4. No cleanup needed — but no isolation either (tests share the same database)

**Problem:** These tests don't use `sqlx::test` so they don't get per-test isolation.
They depend on execution order and clean state. Running them against a shared Neon branch
is fine because CI runs are serialized (one branch per run).

### pt-repo (feature-gated)

Tests use `#[sqlx::test(migrations = "../../migrations")]` which:
- Creates a temporary database per test function
- Runs migrations automatically
- Drops the database after the test

This is the gold standard — fully isolated. Requires `DATABASE_URL` pointing to a
Postgres instance where sqlx can create/drop databases (needs superuser or createdb role).

**Neon consideration:** `sqlx::test` creates databases via `CREATE DATABASE`, which
works on Neon direct endpoints. Pooled endpoints (PgBouncer) may not support DDL.
The direct (non-pooler) endpoint should be used for these tests.

---

## Neon Branching Mechanics

From T-021-01 research and Neon docs:

- **Branch creation:** `neonctl branches create` — creates a copy-on-write branch in <1s
- **Branch deletion:** `neonctl branches delete` — instant
- **Connection strings:** Each branch gets its own direct and pooled endpoints
- **neonctl output:** `--output json` returns structured data with connection URIs
- **API alternative:** REST API at `https://console.neon.tech/api/v2/`

**Secrets needed in GitHub Actions:**
- `NEON_API_KEY` — API token for neonctl authentication
- `NEON_PROJECT_ID` — Neon project identifier

**Free tier limit:** 10 branches. CI creates 1 per run and deletes it after. Only a
problem if multiple CI runs overlap and cleanup fails repeatedly.

---

## neonctl CLI in CI

Options for getting neonctl in GitHub Actions:

1. **npm install:** `npm install -g neonctl` — adds ~5s, well-understood
2. **Direct binary:** Download from GitHub releases — faster but more fragile
3. **GitHub Action:** No official Neon action exists for branch management
4. **Raw API calls:** `curl` against Neon REST API — no dependency, but verbose

neonctl requires `NEON_API_KEY` env var for auth. Branch operations:
```
neonctl branches create --project-id $ID --name ci-run-123 --output json
neonctl branches delete --project-id $ID ci-run-123
```

The JSON output from `branches create` includes the connection URI in the `connection_uri`
field, but extracting it requires `jq` or similar parsing.

---

## Connection String Routing

Two types of tests need different connection strings:

| Test suite | Connection type | Why |
|---|---|---|
| plantastic-api (`#[ignore]`) | Direct or pooled | Just runs queries, either works |
| pt-repo (`#[cfg(feature = "integration")]`) | **Direct only** | `sqlx::test` needs `CREATE DATABASE` (DDL) |

The safest approach: use the **direct** endpoint for `DATABASE_URL` in CI. This works
for both test suites. The pooled endpoint isn't needed — CI tests don't simulate
Lambda concurrency patterns.

---

## Scan Tests (S3 dependency)

6 tests in `scan_test.rs` require both Postgres and S3. These tests use
`plantastic_api::s3::create_s3_client()` which reads AWS config from environment.
Without S3/LocalStack in CI, these tests will fail even with a database.

Options:
- Skip scan tests in CI (they already have distinct `#[ignore]` annotations)
- Add LocalStack to CI (out of scope for this ticket)
- Run only non-S3 integration tests

**Decision point for Design phase:** whether to un-ignore scan tests or only Postgres-only tests.

---

## Justfile Integration Test Recipe

Current `just test-integration`:
```
DATABASE_URL="${DATABASE_URL:-...}" cargo test -p pt-repo --features integration
```

This only runs pt-repo tests. The plantastic-api `#[ignore]` tests are run via:
```
DATABASE_URL=... cargo test -p plantastic-api -- --ignored
```

CI needs to run both.

---

## Cleanup Safety

If CI fails mid-run, the Neon branch must still be deleted. GitHub Actions provides:
- `if: always()` on a step — runs even if previous steps fail
- Composite action with `post` — runs cleanup in post phase
- Separate cleanup cron — sweeps orphaned branches periodically

The `if: always()` approach is simplest and sufficient. A belt-and-suspenders cron
can be added later if orphaned branches become an issue.

---

## Migration Handling

**pt-repo tests:** `sqlx::test` runs migrations automatically (specified in the attribute).
**plantastic-api tests:** `common::setup_test_db()` runs migration SQL files manually.

Both approaches work against a fresh Neon branch because the branch is forked from
the dev branch which already has migrations applied. However, it's cleaner to fork
from a branch that matches the PR's migration state — which means running migrations
on the ephemeral branch from the repo's `migrations/` directory.

**Simpler approach:** Fork from the Neon `main` (or `dev`) branch. The ephemeral branch
inherits the parent's schema. If the PR adds new migrations, run them before tests.

---

## Summary of Findings

| Aspect | Current State | What T-021-02 Changes |
|---|---|---|
| CI database | None | Ephemeral Neon branch per run |
| Integration tests in CI | Skipped | Run against real Postgres |
| pt-repo tests | Feature-gated, manual | Enabled in CI with `--features integration` |
| API tests | `#[ignore]`, manual | Run with `-- --ignored` (Postgres-only subset) |
| Cleanup | N/A | `if: always()` step deletes branch |
| Secrets | AWS, CF tokens | + `NEON_API_KEY`, `NEON_PROJECT_ID` |
| CI budget impact | ~5-7 min | +~30s branch ops, +~60-120s integration tests |
