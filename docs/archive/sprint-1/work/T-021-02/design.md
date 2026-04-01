# Design: T-021-02 CI Neon Branching

## Decision Summary

Use neonctl via npm to create/delete ephemeral Neon branches in GitHub Actions. Run
both pt-repo and plantastic-api integration tests against the direct endpoint. Skip
S3-dependent scan tests (out of scope). Clean up with `if: always()`.

---

## Options Evaluated

### Option A: neonctl CLI via npm (CHOSEN)

Install `neonctl` with `npm install -g neonctl`, use it for branch create/delete.

**Pros:**
- Official CLI, well-maintained, stable JSON output
- npm already available in ubuntu-latest runners
- Simple auth via `NEON_API_KEY` env var
- `--output json` gives structured connection strings

**Cons:**
- Adds npm install step (~3-5s)
- Node.js dependency for a Rust CI job

**Verdict:** Best balance of reliability and simplicity. The Node dependency is trivial
on GitHub Actions runners which already have Node pre-installed.

### Option B: Neon REST API via curl

Call `https://console.neon.tech/api/v2/projects/{id}/branches` directly.

**Pros:**
- Zero dependencies beyond curl/jq
- Full control over request/response

**Cons:**
- Verbose: need to construct JSON payloads, parse responses, handle pagination
- Must manually poll for branch readiness (the API is async)
- Connection string assembly from separate endpoint/role/password fields
- More error-prone, harder to maintain

**Verdict:** Too much shell scripting for a CI step. The complexity isn't worth avoiding
one npm install.

### Option C: Reusable GitHub Action

Write a custom composite action that wraps neonctl.

**Pros:**
- Clean YAML interface
- Reusable across workflows

**Cons:**
- Over-engineered for a single workflow with two steps (create + delete)
- More files to maintain
- Indirection makes debugging harder

**Verdict:** Premature abstraction. Inline steps are easier to understand and modify.

### Option D: Docker Postgres in CI (baseline comparison)

Use `services: postgres` in the GitHub Actions workflow.

**Pros:**
- No external dependency on Neon
- Works offline

**Cons:**
- Adds ~30-60s for container startup
- No PostGIS by default (need custom image or apt-get)
- Diverges from production database (Neon vs Docker Postgres)
- Doesn't validate Neon-specific behaviors (cold start, pooler, SSL)

**Verdict:** Rejected. The whole point of T-021-02 is to use Neon's branching, and
testing against the same provider as production is a design goal.

---

## Key Design Decisions

### 1. Use direct endpoint only

Research found two test patterns:
- pt-repo: `sqlx::test` needs `CREATE DATABASE` — requires direct (non-pooler) endpoint
- plantastic-api: reads `DATABASE_URL`, works with either

**Decision:** Set `DATABASE_URL` to the direct endpoint. This satisfies both test suites.
No need for a separate `TEST_DATABASE_URL` — the standard `DATABASE_URL` that both
test suites already read is sufficient.

### 2. Skip S3-dependent tests

The 6 scan tests in `scan_test.rs` require Postgres + S3. Adding LocalStack to CI is
out of scope for this ticket. These tests will remain `#[ignore]` — they already have
proper scenario ID annotations.

The 10 Postgres-only tests in `crud_test.rs` will be un-ignored. We'll run them with
`cargo test -p plantastic-api -- --ignored --skip scan` to run ignored tests except
scan-related ones.

**Wait** — `--skip` works on test names, and all scan tests are in `scan_test`. We can
use `cargo test -p plantastic-api -- --ignored --skip scan_` to skip tests starting
with `scan_`.

### 3. Branch naming and uniqueness

Branch name: `ci-run-{github.run_id}-{github.run_attempt}`

Including `run_attempt` handles re-runs of the same workflow run. The branch name is
globally unique within the Neon project.

### 4. Migration strategy

The Neon `dev` branch has the production schema. The ephemeral branch inherits it via
copy-on-write. If the PR adds new migrations, they need to run on the ephemeral branch.

**Decision:** Run migrations on the ephemeral branch before tests. Use `just db-migrate`
which applies SQL files via psql. This ensures the branch matches the PR's schema state.

However, `just db-migrate` requires psql, which is available on ubuntu-latest via
the `postgresql-client` package (pre-installed).

Actually, simpler: both test suites run their own migrations:
- pt-repo: `sqlx::test(migrations = ...)` handles it per-test-database
- plantastic-api: `common::setup_test_db()` runs all migration files

So we don't need an explicit migration step. The ephemeral branch just needs to exist
with PostGIS enabled (inherited from parent branch).

### 5. Timeout on branch creation

AC requires "fail CI if branch isn't ready in 30 seconds." neonctl blocks until the
branch is ready by default. We'll add a `timeout` wrapper: `timeout 30 neonctl branches create ...`.

### 6. CI step ordering

```
[existing] checkout → setup-rust → cache → install-just
[new]      install neonctl → create branch → export DATABASE_URL
[existing] fmt-check → lint
[new]      test (now with DATABASE_URL) → test-integration
[existing] scenarios
[new]      cleanup branch (if: always)
```

Linting and formatting don't need the database, so they run first (fail-fast). Tests
and integration tests run after the branch is ready.

### 7. Separate integration test step

Add a new step `Integration tests` after the existing `Test` step. This keeps the
existing unit test step unchanged and makes integration test results visible as a
distinct step in the CI UI.

```yaml
- name: Integration tests
  run: |
    cargo test -p pt-repo --features integration
    cargo test -p plantastic-api -- --ignored --skip scan_
  env:
    DATABASE_URL: ${{ steps.neon-branch.outputs.database_url }}
```

---

## Rejected Alternatives

### Separate CI job for integration tests

Could run integration tests in a parallel job. Rejected because:
- Doubles the rust compilation time (cache helps but not 100%)
- Adds complexity for marginal benefit
- Single job is simpler to reason about

### Running all tests with `-- --include-ignored`

This would run both normal and ignored tests in one pass. Rejected because:
- Can't selectively skip scan tests
- Mixes unit and integration test output
- `cargo test --workspace -- --include-ignored` would try to run scan tests too

---

## Risk Assessment

| Risk | Likelihood | Mitigation |
|---|---|---|
| Branch creation fails (Neon outage) | Low | CI fails explicitly; unit tests still ran |
| Orphaned branches (cleanup fails) | Low | `if: always()` + manual sweep if needed |
| Free tier branch limit (10) | Low | One branch per run, deleted immediately |
| Integration tests slow down CI | Medium | Budget check: ~2 min integration + ~10s branching |
| neonctl breaking change | Low | Pin to specific version |

---

## Expected CI Budget Impact

| Step | Time |
|---|---|
| npm install neonctl | ~3s |
| Branch creation | ~2-5s |
| pt-repo integration tests | ~30-60s |
| plantastic-api integration tests | ~30-60s |
| Branch deletion | ~1-2s |
| **Total overhead** | **~70-130s** |

Current CI: ~5-7 min. With integration tests: ~7-9 min. Within the 10-min budget.
