# T-017-02 Review: Neon PostGIS + S3 CORS + Secrets Wiring

## Summary

This ticket wires the deployed Lambda (from T-017-01) to managed PostgreSQL and S3
with proper secrets management. The original ticket referenced Railway, but per the
project decision (2026-03-31), Neon replaces Railway as the managed PostgreSQL provider.

All changes are infrastructure configuration, developer tooling, and documentation.
No Rust application code was modified.

---

## Files Created

| File | Purpose |
|---|---|
| `.doppler.yaml` | Doppler CLI project/config pointer for local dev |
| `scripts/migrate.sh` | Migration runner wrapping sqlx-cli with validation |
| `scripts/verify-deploy.sh` | Deployment verification (health, CRUD round-trip, timing) |
| `docs/active/work/T-017-02/setup-neon.md` | Step-by-step Neon provisioning guide |
| `docs/active/work/T-017-02/research.md` | Research artifact |
| `docs/active/work/T-017-02/design.md` | Design artifact |
| `docs/active/work/T-017-02/structure.md` | Structure artifact |
| `docs/active/work/T-017-02/plan.md` | Plan artifact |
| `docs/active/work/T-017-02/progress.md` | Implementation progress tracker |

## Files Modified

| File | Change |
|---|---|
| `infra/sst.config.ts` | Added CORS config to S3 Uploads bucket (GET/PUT/POST, all origins, 1hr max age) |
| `.env.example` | Added Neon connection string example and Doppler usage note |
| `justfile` | Added `migrate` (Doppler) and `migrate-direct` (sqlx-cli) recipes |
| `tests/scenarios/src/progress.rs` | Claimed milestone: "Neon PostGIS + S3 CORS + secrets wiring" |

---

## Acceptance Criteria Status

### Railway→Neon PostGIS
- [x] PostGIS extension: handled by migration 001 (`CREATE EXTENSION IF NOT EXISTS postgis`)
- [x] Migrations: `scripts/migrate.sh` + `just migrate` / `just migrate-direct` ready
- [x] DATABASE_URL management: Doppler (dev) + SST Secret/SSM (prod) documented
- [ ] **Manual step required:** Create Neon project, run migrations, set secrets
  (see `setup-neon.md`)

### S3
- [x] SST creates S3 bucket (from T-017-01)
- [x] Bucket name available to Lambda via S3_BUCKET env var (from T-017-01)
- [x] CORS configured for browser direct uploads

### Secrets
- [x] Doppler project config: `.doppler.yaml` created
- [x] `doppler run` documented for local development
- [x] SST Secret for DATABASE_URL (prod via SSM) already existed from T-017-01
- [x] `sst secret set` command documented in setup guide
- [ ] **Manual step required:** Create Doppler project, set secrets

### Verification
- [x] `scripts/verify-deploy.sh` tests health, project CRUD, and cold start timing
- [ ] **Manual step required:** Run verify-deploy.sh against deployed Lambda

---

## Scenario Dashboard: Before and After

**Before:** 58.0 min / 240.0 min (24.2%), 13/22 milestones
**After:** 58.0 min / 240.0 min (24.2%), 14/23 milestones

Effective savings unchanged — this is infrastructure wiring, not a user-facing
capability. The milestone count increased (14/23 delivered). No regressions.

---

## Quality Gate

```
just check — ALL GATES PASSED
  - fmt-check: pass
  - lint (clippy strict): pass
  - test: 106 passed, 0 failed, 10 ignored (all ignored tests have scenario IDs)
  - scenarios: 8 pass, 0 fail, 9 not implemented, 0 blocked
```

---

## Test Coverage

No new Rust application code was written, so no new tests. The existing test suite
(106 tests) passes without modification. The 10 ignored integration tests all have
proper scenario ID annotations per CLAUDE.md rule #4.

The verification script (`verify-deploy.sh`) serves as the integration test for this
ticket — it exercises the deployed Lambda → Neon → S3 path end-to-end.

---

## Open Concerns

1. **Manual provisioning required.** Neon project creation, Doppler project setup,
   and secret configuration are manual steps documented in `setup-neon.md`. These
   must be completed before the Lambda can actually connect to a database.

2. **Neon cold-start hang risk.** Memory notes a known issue with sqlx/Neon on Lambda
   cold start. Mitigation is `connect_timeout=5` in the connection string (documented
   in setup guide). If this proves insufficient, a follow-up ticket should add retry
   logic to `create_pool()`.

3. **S3 CORS wildcard origin.** CORS is currently set to `allowOrigins: ["*"]` for
   development convenience. Should be locked down to the actual frontend domain before
   production launch.

4. **Ticket title mismatch.** Ticket says "railway-s3-secrets" but implementation
   targets Neon per project decision. Title is cosmetic — the work matches the
   current architectural direction.

5. **Existing `db-migrate` recipe.** The justfile already had a `db-migrate` recipe
   using `psql`. The new `migrate` and `migrate-direct` recipes use `sqlx-cli` instead.
   Both coexist. The psql-based recipe may be removed in a future cleanup if sqlx-cli
   becomes the standard.
