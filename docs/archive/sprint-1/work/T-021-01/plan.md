# Plan: T-021-01 Neon Provisioning

## Step 1: Write verification script

**File:** `scripts/verify-neon.sh`

Write the script that validates all acceptance criteria against a live Neon instance. This is the acceptance test — everything else is in service of making this script pass.

**Checks:**
1. Direct endpoint: `psql` connection + `SELECT 1`
2. PostGIS: `SELECT PostGIS_Version()`
3. Tables: query `pg_tables` for tenants, projects, zones, materials, tier_assignments, plants
4. Spatial roundtrip: insert zone polygon → fetch → verify coordinates → delete
5. Pooled endpoint: `psql` connection + query

**Verification:** Script runs without errors against a working database (test against local Docker first for syntax/logic).

## Step 2: Add Justfile recipe

**File:** `justfile`

Add `verify-neon` recipe that runs the verification script. Takes optional connection string arguments or reads from environment.

**Verification:** `just verify-neon` prints usage when no args provided.

## Step 3: Write cost analysis

**File:** `docs/active/work/T-021-01/cost-analysis.md`

Document Neon pricing for the ticket's cost verification AC:
- Free tier: 0.5 GiB storage, 191 compute-hours/month, 1 branch
- Launch plan: $19/mo, 10 GiB storage, 300 compute-hours, 10 branches
- Scale-to-zero behavior: 5-min default suspend timeout
- Expected cost for Plantastic (dev-stage, low traffic): free tier sufficient for now
- Railway comparison: $5/mo grandfathered vs Neon free/$19

**Verification:** Document covers all cost-related ACs.

## Step 4: Provision Neon project

**This is a manual/CLI step requiring user interaction.**

Document the provisioning commands in `docs/active/work/T-021-01/provisioning-log.md`:

```
neonctl projects create --name plantastic --region aws-us-west-2 --pg-version 17
```

Then:
- Enable PostGIS: `CREATE EXTENSION IF NOT EXISTS postgis;`
- Run migrations: `DATABASE_URL="<direct-url>" just migrate-direct`
- Record both connection strings (direct + pooled)

**Verification:** `scripts/verify-neon.sh <direct> <pooled>` passes all checks.

**Note:** Since this requires actual Neon credentials and network access, the provisioning-log.md will document the exact commands to run. The user executes them. The script validates the result.

## Step 5: Update setup guide

**File:** `docs/active/work/T-017-02/setup-neon.md`

- Add `neonctl` CLI installation: `brew install neonctl` or `npm i -g neonctl`
- Add CLI commands for project creation alongside console instructions
- Fix SST section: pooled URL should include `sslnegotiation=direct`
- Add reference to `scripts/verify-neon.sh`

**Verification:** Guide is accurate relative to actual provisioning experience.

## Step 6: Configure secrets (manual, documented)

Document the commands to set secrets:

**Doppler:**
```
doppler secrets set DATABASE_URL "<neon-direct-url>" -p plantastic -c dev
```

**SSM (SST):**
```
cd infra && npx sst secret set DatabaseUrl "<neon-pooled-url>" --stage dev
```

**Verification:** `doppler run -- env | grep DATABASE_URL` shows Neon URL. SST deploy uses the secret.

## Step 7: Run quality gate

Run `just check` (fmt + lint + test + scenarios) to verify no regressions. This ticket adds no Rust code, so the gate should pass unchanged. Run scenario dashboard before and after.

**Verification:** `just check` passes. Scenario dashboard unchanged.

## Testing Strategy

This ticket is infrastructure provisioning — no unit tests. Testing is:

1. **Verification script** (`scripts/verify-neon.sh`) — the primary acceptance test
2. **Quality gate** (`just check`) — no regressions
3. **Scenario dashboard** (`just scenarios`) — unchanged baseline

The verification script is the test. If it passes against the provisioned Neon instance, all ACs are met.

## Commit Plan

1. `scripts/verify-neon.sh` + justfile recipe + cost analysis (code deliverables)
2. `provisioning-log.md` + setup guide update (documentation)
