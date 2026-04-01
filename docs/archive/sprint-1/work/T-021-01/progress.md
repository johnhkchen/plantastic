# Progress: T-021-01 Neon Provisioning

## Completed

### Step 1: Verification script
- Created `scripts/verify-neon.sh` — validates all 6 ACs against live Neon
- Checks: direct connectivity, PostGIS, 6 tables, spatial roundtrip, pooled connectivity, pooled query
- Script is executable, uses psql, exits non-zero on any failure

### Step 2: Justfile recipe
- Added `verify-neon` recipe to justfile
- Usage: `just verify-neon <direct-url> <pooled-url>`

### Step 3: Cost analysis
- Created `docs/active/work/T-021-01/cost-analysis.md`
- Covers Free/Launch/Scale tiers, Railway comparison, scale-to-zero behavior
- Recommendation: Free tier now, Launch when production traffic arrives

### Step 4: Provisioning log
- Created `docs/active/work/T-021-01/provisioning-log.md`
- Step-by-step neonctl commands with placeholders for actual values
- Checklist for user to complete after executing

### Step 5: Setup guide update
- Updated `docs/active/work/T-017-02/setup-neon.md`:
  - Added neonctl CLI as prerequisite and primary creation method
  - Fixed SST connection strings: added `sslnegotiation=direct` and `statement_cache_size=0`
  - Fixed database name: `neondb` → `plantastic`
  - Added verification step before deploy (references `scripts/verify-neon.sh`)
  - Updated SST notes to reference T-020-02 retry logic instead of `connect_timeout` param

## Remaining

### Step 6: Actual Neon provisioning (manual)
- Requires user to run `neonctl` commands from provisioning-log.md
- Requires Neon account authentication
- Requires Doppler and SST secret configuration
- Cannot be automated in this session — documented for user execution

### Step 7: Quality gate
- Run `just check` to verify no regressions

## Deviations from Plan

None. The plan anticipated that actual provisioning (Step 4 in plan) requires user interaction with external services. All automatable deliverables (verification script, cost docs, provisioning guide, setup guide updates) are complete.
