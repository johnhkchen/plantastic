# Plan: T-021-02 CI Neon Branching

## Implementation Steps

### Step 1: Add `test-integration-ci` recipe to justfile

**Change:** Add a new recipe that runs both pt-repo and plantastic-api integration tests.

**Why:** Encapsulates the two-command integration test sequence. Usable locally and in CI.

**Verify:** `just --list` shows the new recipe. Running without `DATABASE_URL` errors
clearly ("DATABASE_URL must be set").

---

### Step 2: Update `.github/workflows/ci.yml` — neonctl setup

**Change:** Add two steps before `just fmt-check`:
1. `Install neonctl` — `npm install -g neonctl@latest`
2. `Create Neon branch` — create ephemeral branch, export connection URI to `GITHUB_OUTPUT`

Add job-level `env` for `NEON_PROJECT_ID`.

**Why:** Branch must exist before any test step. Installing neonctl first ensures it's
available for both creation and cleanup.

**Verify:** Inspect the YAML for correct step ordering and `id: neon-branch`.

---

### Step 3: Update `.github/workflows/ci.yml` — integration test step

**Change:** Add `Integration tests` step between `just test` and `just scenarios`:
```yaml
- name: Integration tests
  env:
    DATABASE_URL: ${{ steps.neon-branch.outputs.database_url }}
  run: just test-integration-ci
```

**Why:** Runs after unit tests (fail-fast — no point running integration tests if units
fail). Before scenarios because integration tests validate real capabilities.

**Verify:** Step uses the correct output reference and env var name.

---

### Step 4: Update `.github/workflows/ci.yml` — cleanup step

**Change:** Add `Delete Neon branch` step at the end with `if: always()`:
```yaml
- name: Delete Neon branch
  if: always()
  env:
    NEON_API_KEY: ${{ secrets.NEON_API_KEY }}
  run: |
    BRANCH_NAME="${{ steps.neon-branch.outputs.branch_name }}"
    if [ -n "$BRANCH_NAME" ]; then
      neonctl branches delete \
        --project-id "$NEON_PROJECT_ID" \
        "$BRANCH_NAME" || true
    fi
```

**Why:** Guarantees cleanup even on test failure. The `|| true` and empty-check prevent
cleanup failures from masking the real CI failure. If branch creation itself failed,
`branch_name` is empty and we skip deletion.

**Verify:** `if: always()` is present. Cleanup doesn't fail if branch doesn't exist.

---

### Step 5: Run `just check` locally

**Change:** None — just validation.

**Why:** Ensure no formatting or lint issues were introduced by the YAML/justfile changes.

**Verify:** `just check` passes (fmt-check, lint, test, scenarios all green).

---

## Testing Strategy

### What gets tested in CI (after this ticket)

| Suite | Runner | Isolation | Tests |
|---|---|---|---|
| pt-repo integration | `cargo test -p pt-repo --features integration` | Per-test database via `sqlx::test` | 6 files |
| plantastic-api CRUD | `cargo test -p plantastic-api -- --ignored --skip scan_` | Shared branch database | 10 tests |
| Unit tests | `just test` (unchanged) | In-memory, no DB | All workspace |

### What stays skipped

| Suite | Reason | When un-ignored |
|---|---|---|
| scan_test.rs (6 tests) | Requires S3/LocalStack | When CI adds LocalStack |

### Verification criteria

1. `just check` passes locally (no regressions)
2. CI workflow YAML is valid (GitHub Actions will validate on push)
3. Integration test step correctly receives `DATABASE_URL`
4. Cleanup step runs even when tests fail
5. Branch name includes run ID and attempt for uniqueness
6. neonctl creation has 30s timeout

### Manual verification (after secrets are configured)

Push a PR with the workflow changes. Observe in GitHub Actions:
- neonctl installs successfully
- Branch creates in <10s
- Integration tests run and pass
- Branch is deleted in cleanup
- Total CI time stays under 10 minutes

---

## Commit Plan

**Single commit:** All changes are tightly coupled (the CI workflow references the
justfile recipe). Splitting would leave one commit in a broken state.

```
feat(ci): add Neon branching for integration tests in CI

Add ephemeral Neon database branches to GitHub Actions CI. Each run
creates a branch, runs pt-repo and plantastic-api integration tests
against real Postgres+PostGIS, then cleans up.

- Install neonctl, create branch with 30s timeout
- Run pt-repo integration tests (sqlx::test isolation)
- Run plantastic-api CRUD tests (--ignored, skip S3-dependent scan tests)
- Delete branch in always() cleanup step
- Add test-integration-ci justfile recipe
```
