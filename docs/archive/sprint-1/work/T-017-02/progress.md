# T-017-02 Progress

## Step 1: Doppler Configuration — DONE
- Created `.doppler.yaml`
- Updated `.env.example` with Neon connection string format

## Step 2: S3 CORS in SST — DONE
- Added CORS configuration to Uploads bucket in `infra/sst.config.ts`

## Step 3: Migration Script + Justfile — DONE
- Created `scripts/migrate.sh`
- Added `migrate` and `migrate-direct` recipes to justfile

## Step 4: Verification Script — DONE
- Created `scripts/verify-deploy.sh`

## Step 5: Neon Setup Guide — DONE
- Created `docs/active/work/T-017-02/setup-neon.md`

## Step 6: Claim Milestone — DONE
- Added milestone in `tests/scenarios/src/progress.rs`

## Step 7: Quality Gate — DONE
- `just check` passes

## Deviations from Plan
- Justfile: kept existing `db-migrate` recipe, added `migrate` (Doppler) and
  `migrate-direct` (sqlx-cli) as alternatives. Did not remove existing recipe
  to avoid breaking existing workflows.
