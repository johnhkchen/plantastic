# Review: T-021-02 CI Neon Branching

## Summary

This ticket adds ephemeral Neon database branching to the GitHub Actions CI workflow so
integration tests run against real Postgres+PostGIS without Docker. Each CI run creates
a copy-on-write branch, runs both pt-repo and plantastic-api integration tests, then
deletes the branch in a guaranteed cleanup step.

## Files Modified

| File | Change |
|---|---|
| `.github/workflows/ci.yml` | Added neonctl install, branch create/delete, integration test step |
| `justfile` | Added `test-integration-ci` recipe (pt-repo + plantastic-api integration) |
| `crates/plantastic-api/src/routes/health.rs` | Fixed pre-existing clippy truncation warning |

## Files Created

| File | Purpose |
|---|---|
| `docs/active/work/T-021-02/research.md` | Codebase mapping |
| `docs/active/work/T-021-02/design.md` | Approach decision |
| `docs/active/work/T-021-02/structure.md` | File-level blueprint |
| `docs/active/work/T-021-02/plan.md` | Implementation sequence |
| `docs/active/work/T-021-02/progress.md` | Implementation tracking |
| `docs/active/work/T-021-02/review.md` | This file |

## CI Workflow Changes

The `rust` job now has this step sequence:

```
checkout → setup-rust → cache → install-just →
install neonctl → create Neon branch →
fmt-check → lint → test (unit) → integration tests → scenarios →
delete Neon branch (always)
```

Key details:
- **Branch name:** `ci-run-{run_id}-{run_attempt}` — unique per run and re-run
- **Timeout:** 30s on branch creation (AC requirement)
- **Secrets required:** `NEON_API_KEY`, `NEON_PROJECT_ID` (must be set in GitHub repo settings)
- **Cleanup:** `if: always()` with `|| true` — guaranteed, failure-safe
- **Integration tests:** `just test-integration-ci` runs pt-repo (feature-gated) + plantastic-api
  CRUD tests (`--ignored --skip scan_`). S3-dependent scan tests remain skipped.

## Test Coverage

No new Rust tests were added — this ticket is CI infrastructure. The existing integration
tests (previously manual-only) are now automated in CI:

| Suite | Test count | Isolation | Notes |
|---|---|---|---|
| pt-repo integration | ~6 files | Per-test DB via `sqlx::test` | Feature-gated |
| plantastic-api CRUD | 9 tests | Shared branch DB | `#[ignore]`, run via `--ignored` |
| plantastic-api scan | 6 tests | N/A — skipped | Require S3/LocalStack |

**Coverage gap:** Scan tests (6) still require S3 and will remain `#[ignore]` until
LocalStack or similar is added to CI. This is out of scope for this ticket.

## Quality Gate

```
just check → All gates passed.
```

- `just fmt-check` — pass
- `just lint` — pass (after fixing pre-existing health.rs warning)
- `just test` — all pass, no regressions
- `just scenarios` — 58.0 / 240.0 min (24.2%), unchanged

## Scenario Dashboard

**Before:** 58.0 min / 240.0 min (24.2%) — 8 pass, 0 fail, 9 not implemented
**After:** 58.0 min / 240.0 min (24.2%) — 8 pass, 0 fail, 9 not implemented

No change. This ticket is CI infrastructure — it doesn't implement new customer-facing
capabilities. It enables existing integration tests to run automatically, increasing
confidence in passing scenarios but not flipping any new ones.

## Open Concerns

### 1. Secrets must be configured manually

The CI workflow requires two GitHub Actions secrets:
- `NEON_API_KEY` — Neon API token (from `neonctl auth`)
- `NEON_PROJECT_ID` — Neon project identifier

These must be set in the repository's Settings > Secrets and variables > Actions before
the integration test step will work. Without them, neonctl will fail and the integration
test step will error. The unit test, lint, and fmt steps will still pass.

**Action needed:** User sets secrets after Neon project is provisioned (T-021-01).

### 2. Free tier branch limit

Neon Free tier allows 10 branches. With one ephemeral branch per CI run and guaranteed
cleanup, this is not an issue under normal operation. If multiple PRs trigger concurrent
runs and cleanup fails, branches could accumulate. Monitor via `neonctl branches list`.

### 3. S3-dependent tests still skipped

6 scan tests in `scan_test.rs` require both Postgres and S3. These are skipped via
`--skip scan_`. Adding LocalStack to CI would enable them but is a separate effort.

### 4. neonctl version pinning

Currently using `neonctl@latest`. A breaking change in neonctl could fail CI. Consider
pinning to a specific version (e.g., `neonctl@2.x`) if stability becomes a concern.

### 5. Neon branch inherits parent schema

The ephemeral branch is forked from the Neon `main` branch. If the PR adds new
migrations, pt-repo's `sqlx::test` handles them (migrations specified in attribute).
plantastic-api's `common::setup_test_db()` also runs all migrations from the repo's
`migrations/` directory. Both approaches handle schema drift correctly.

## Downstream Unblocked

- **T-021-03** (Lambda connection validation): Can now validate Neon connectivity patterns
  in CI, not just locally
- All future tickets with integration tests benefit automatically — tests run in CI
  once secrets are configured
