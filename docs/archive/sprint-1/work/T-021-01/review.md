# Review: T-021-01 Neon Provisioning

## Summary

This ticket delivers the tooling, documentation, and configuration updates needed to provision Neon as the managed PostgreSQL provider. No Rust application code was changed — the codebase was already Neon-ready from T-020-02.

## Files Created

| File | Purpose |
|---|---|
| `scripts/verify-neon.sh` | Validation script — 6 checks against live Neon instance |
| `docs/active/work/T-021-01/research.md` | Codebase mapping for Neon provisioning |
| `docs/active/work/T-021-01/design.md` | Approach decision: neonctl CLI + verification script |
| `docs/active/work/T-021-01/structure.md` | File-level change blueprint |
| `docs/active/work/T-021-01/plan.md` | Implementation step sequence |
| `docs/active/work/T-021-01/progress.md` | Implementation tracking |
| `docs/active/work/T-021-01/cost-analysis.md` | Neon pricing analysis vs Railway |
| `docs/active/work/T-021-01/provisioning-log.md` | Step-by-step commands for user execution |

## Files Modified

| File | Change |
|---|---|
| `justfile` | Added `verify-neon` recipe |
| `docs/active/work/T-017-02/setup-neon.md` | Added neonctl CLI, fixed pooled URL params, added verification step |

## What the Verification Script Checks

1. **Direct endpoint connectivity** — psql connects to non-pooled endpoint
2. **PostGIS extension** — `PostGIS_Version()` returns a version
3. **All 6 tables** — materials, plants, projects, tenants, tier_assignments, zones
4. **Spatial roundtrip** — insert tenant → project (POINT) → zone (POLYGON), fetch geometry back, verify coordinates, delete test data
5. **Pooled endpoint connectivity** — psql connects to `-pooler` endpoint
6. **Pooled endpoint query** — query via PgBouncer returns correct table count

## Test Coverage

This ticket is infrastructure provisioning. No Rust code was added or modified, so no unit tests apply.

**Testing is via:**
- `scripts/verify-neon.sh` — the acceptance test for all ACs (run against live Neon)
- `just check` — quality gate passes with no regressions

## Quality Gate

```
just check → All gates passed.
```

- fmt-check: pass
- lint (clippy strict): pass
- test (workspace): all pass, no new ignored tests
- scenarios: dashboard unchanged (no scenario regressions)

## Scenario Dashboard

No change — this ticket is infrastructure provisioning. No scenarios were expected to flip. The scenario dashboard baseline is unchanged before and after.

## Open Concerns

### 1. Actual provisioning requires user execution

The provisioning-log.md documents all commands, but actual Neon project creation, Doppler secret setting, and SSM configuration require user interaction with external services (Neon auth, Doppler auth, AWS credentials). These cannot be automated in this session.

**Action needed:** User runs the commands in `provisioning-log.md`, then runs `scripts/verify-neon.sh` to validate.

### 2. Database rename (neondb → plantastic)

Neon creates a default database named `neondb`. The provisioning log includes an `ALTER DATABASE` to rename to `plantastic` for consistency with the codebase. This is a one-time manual step.

### 3. Free tier branch limits for CI

The Free tier allows 10 branches. T-021-02 creates ephemeral branches per CI run. If CI runs frequently with slow cleanup, branch count could approach the limit. The cost analysis recommends monitoring and upgrading to Launch ($19/mo) if this becomes an issue.

### 4. Scale-to-zero wake time

Neon compute wakes in 0.5-3s (typical) to 8s (worst case). Combined with Lambda cold start, first-request latency could be 5-10s. T-020-02's retry logic (15s timeout, 3 retries) should handle this, but T-021-03 will validate with real measurements.

## Downstream Unblocked

- **T-021-02** (CI Neon branching): needs the provisioned Neon project + API token
- **T-021-03** (Lambda connection validation): needs Lambda deployed with Neon pooled endpoint

Both can proceed once the user completes the provisioning steps documented here.
