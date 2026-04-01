# T-038-03 Plan: db-scenario-recipe

## Step 1: Fix port references in justfile

Change port 5432 → 5433 in three existing recipes:
- `test-integration`: default DATABASE_URL on line 75
- `dev-db`: printed URL on line 164
- `dev-reset`: printed URL on line 192

**Verify:** `grep 5432 justfile` returns no hits (all should now be 5433).

## Step 2: Fix port references in .env.example

Change DATABASE_URL and TEST_DATABASE_URL from port 5432 to 5433.

**Verify:** `grep 5432 .env.example` returns no hits for connection strings.

## Step 3: Add `scenarios-db` recipe to justfile

Add the recipe in the Scenarios section, after the existing `scenarios` recipe. The recipe:
1. Checks `docker` command exists, prints helpful error if not
2. Runs `docker compose up -d db`
3. Loops on `docker compose exec db pg_isready` (max 30 attempts, 1s sleep)
4. Sets `DATABASE_URL=postgres://plantastic:plantastic@localhost:5433/plantastic`
5. Runs `cargo run -p pt-scenarios`

**Verify:** `just --list` shows `scenarios-db` recipe with description.

## Step 4: Update CLAUDE.md

Add `just scenarios-db` to the Key Commands block.

**Verify:** Read CLAUDE.md, confirm entry is present.

## Step 5: Run `just scenarios` (no DB)

Confirm it still works and infrastructure scenarios show BLOCKED. No regression.

## Step 6: Run `just fmt-check` and `just lint`

Ensure no formatting or lint issues were introduced (unlikely since no Rust changes, but good hygiene).

## Testing strategy

This ticket modifies only tooling (justfile, .env.example, CLAUDE.md). There are no Rust code changes, so:
- No new unit tests needed
- No new integration tests needed
- Verification is functional: run `just scenarios` and confirm no regression
- Full verification of `just scenarios-db` requires Docker (manual test)
- `just check` should pass (no Rust changes to break)
