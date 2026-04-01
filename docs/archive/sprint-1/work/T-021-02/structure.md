# Structure: T-021-02 CI Neon Branching

## Files Modified

### `.github/workflows/ci.yml`

The primary change. Add steps to the `rust` job:

**Before `just fmt-check`:**
```yaml
- name: Install neonctl
  run: npm install -g neonctl@latest

- name: Create Neon branch
  id: neon-branch
  env:
    NEON_API_KEY: ${{ secrets.NEON_API_KEY }}
  run: |
    # Create ephemeral branch, capture connection URI
    BRANCH_NAME="ci-run-${{ github.run_id }}-${{ github.run_attempt }}"
    BRANCH_JSON=$(timeout 30 neonctl branches create \
      --project-id "$NEON_PROJECT_ID" \
      --name "$BRANCH_NAME" \
      --output json)
    # Extract direct connection URI from JSON
    DATABASE_URL=$(echo "$BRANCH_JSON" | jq -r '.connection_uri')
    echo "database_url=$DATABASE_URL" >> "$GITHUB_OUTPUT"
    echo "branch_name=$BRANCH_NAME" >> "$GITHUB_OUTPUT"
```

**After `just scenarios`:**
```yaml
- name: Delete Neon branch
  if: always()
  env:
    NEON_API_KEY: ${{ secrets.NEON_API_KEY }}
  run: |
    neonctl branches delete \
      --project-id "$NEON_PROJECT_ID" \
      "${{ steps.neon-branch.outputs.branch_name }}"
```

**New step between `Test` and `Scenarios`:**
```yaml
- name: Integration tests
  env:
    DATABASE_URL: ${{ steps.neon-branch.outputs.database_url }}
  run: |
    cargo test -p pt-repo --features integration
    cargo test -p plantastic-api -- --ignored --skip scan_
```

**Environment variables added to job level:**
```yaml
env:
  NEON_PROJECT_ID: ${{ secrets.NEON_PROJECT_ID }}
```

### `justfile`

Add a new recipe for CI integration tests that combines both test suites:

```just
# Run integration tests against Neon branch (used by CI)
test-integration-ci:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Running pt-repo integration tests..."
    cargo test -p pt-repo --features integration 2>&1
    echo ""
    echo "Running plantastic-api integration tests (Postgres-only)..."
    cargo test -p plantastic-api -- --ignored --skip scan_ 2>&1
    echo ""
    echo "Integration tests passed."
```

Update the existing `test-integration` recipe comment to reference the CI recipe.

---

## Files NOT Modified

| File | Reason |
|---|---|
| `crates/pt-repo/Cargo.toml` | `integration` feature already exists |
| `crates/plantastic-api/Cargo.toml` | No feature flag needed; tests use `#[ignore]` |
| `crates/plantastic-api/tests/common/mod.rs` | Already reads `DATABASE_URL` |
| `crates/plantastic-api/tests/crud_test.rs` | Tests stay `#[ignore]`; CI runs with `--ignored` |
| `crates/plantastic-api/tests/scan_test.rs` | Stays ignored; skipped via `--skip scan_` |
| `scripts/verify-neon.sh` | Used for manual verification, not CI |
| `.env.example` | Already documents `DATABASE_URL` and `TEST_DATABASE_URL` |

---

## Module Boundaries

No new Rust modules or crates. This ticket is purely CI infrastructure:

```
.github/workflows/ci.yml  ← primary change (workflow steps)
justfile                   ← convenience recipe for local + CI use
```

---

## Secret Requirements

Two new GitHub Actions secrets (set by user in repo settings):

| Secret | Value | Source |
|---|---|---|
| `NEON_API_KEY` | Neon API token | `neonctl auth` → copy token |
| `NEON_PROJECT_ID` | Neon project ID | `neonctl projects list --output json` |

---

## Step Ordering Within CI

```
1. checkout
2. setup-rust-toolchain (stable + clippy, rustfmt)
3. rust-cache
4. install-just
5. install neonctl              ← NEW
6. create Neon branch           ← NEW (id: neon-branch)
7. just fmt-check               (existing)
8. just lint                    (existing)
9. just test                    (existing — unit tests)
10. integration tests           ← NEW (uses DATABASE_URL from step 6)
11. just scenarios              (existing)
12. delete Neon branch          ← NEW (if: always)
```

Format and lint run before integration tests for fail-fast behavior. If formatting
or linting fails, we skip the expensive integration tests but still clean up the branch.

---

## Data Flow

```
GitHub Secrets
    │
    ├── NEON_API_KEY ──→ neonctl auth
    └── NEON_PROJECT_ID ──→ neonctl --project-id
                                │
                          branches create
                                │
                          connection_uri (JSON output)
                                │
                     ┌──────────┴──────────┐
                     │                     │
              GITHUB_OUTPUT           DATABASE_URL env
              (branch_name)                │
                     │            ┌────────┴────────┐
                     │            │                 │
                     │    pt-repo tests      API tests
                     │    (sqlx::test)    (--ignored --skip)
                     │
              branches delete
              (if: always)
```

---

## Verification

After implementation, verify:

1. **CI passes:** Push a PR, observe integration tests running
2. **Branch cleanup:** After CI completes, check `neonctl branches list` — no orphans
3. **Fail-fast:** Break formatting, observe integration tests skipped but branch cleaned up
4. **Budget:** Total CI time stays under 10 minutes
5. **Secrets:** CI fails gracefully if secrets are missing (neonctl errors clearly)
