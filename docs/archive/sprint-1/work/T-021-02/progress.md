# Progress: T-021-02 CI Neon Branching

## Completed

### Step 1: Add `test-integration-ci` recipe to justfile
- Added recipe at `justfile:80-91`
- Runs `cargo test -p pt-repo --features integration` then
  `cargo test -p plantastic-api -- --ignored --skip scan_`
- Requires `DATABASE_URL` env var (no default — CI must set it)

### Step 2: Update CI workflow — neonctl setup
- Added `Install neonctl` step (npm install -g neonctl@latest)
- Added `Create Neon branch` step with `id: neon-branch`
  - Branch name: `ci-run-{run_id}-{run_attempt}`
  - 30s timeout on creation
  - Exports `database_url` and `branch_name` to `GITHUB_OUTPUT`
- Added job-level `NEON_PROJECT_ID` env var from secrets

### Step 3: Update CI workflow — integration test step
- Added `Integration tests` step between `Test` and `Scenarios`
- Uses `DATABASE_URL` from `steps.neon-branch.outputs.database_url`
- Runs `just test-integration-ci`

### Step 4: Update CI workflow — cleanup step
- Added `Delete Neon branch` step with `if: always()`
- Guards against empty branch name (if creation failed)
- Uses `|| true` to prevent cleanup failure from masking test failures

### Step 5: Run `just check` locally
- `just fmt-check` — pass
- `just lint` — fixed pre-existing clippy truncation warning in health.rs
- `just test` — all pass (197 tests + 15 ignored)
- `just scenarios` — 58.0 / 240.0 min (24.2%), no regressions

## Deviations from Plan

### Pre-existing lint fix
Found `clippy::cast_possible_truncation` warning in
`crates/plantastic-api/src/routes/health.rs:49`. Added `#[allow]` annotation.
Per CLAUDE.md rule 6, fixed in-place rather than filing a separate ticket.

## Remaining

None — all plan steps completed.
