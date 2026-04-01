# Design: T-021-01 Neon Provisioning

## Problem

Replace Railway with Neon as the production PostgreSQL provider. The codebase is already Neon-ready (T-020-02 connection hardening, `.env.example` templates, setup guide). This ticket is infrastructure provisioning + validation, not application code.

## Approaches Considered

### A. Manual Console Provisioning + Verification Script

Provision via Neon Console (web UI), then write a `scripts/verify-neon.sh` script that validates the entire setup: connects to both endpoints, checks PostGIS, runs a spatial roundtrip, verifies all tables exist.

**Pros:** Simple, auditable, the verification script is reusable for T-021-03 and future validation. Console UI is most reliable for one-time setup.
**Cons:** Manual step cannot be automated; but this is a one-time operation.

### B. neonctl CLI Provisioning + Verification Script

Use `neonctl` CLI to create the project, enable PostGIS, and extract connection strings programmatically. Same verification script as (A).

**Pros:** Reproducible command sequence. CLI output can be documented step-by-step.
**Cons:** `neonctl` requires auth setup. For a one-time operation, the overhead of CLI setup vs. console is marginal. PostGIS extension still requires a SQL command after project creation.

### C. Terraform/Pulumi IaC

Define the Neon project as infrastructure-as-code alongside the SST config.

**Pros:** Fully reproducible, version-controlled.
**Cons:** Massive overhead for a single database project. Neon's Terraform provider is still maturing. The rest of the infra uses SST (not Terraform). Overkill — we provision one Neon project, not a fleet.

## Decision: Approach B — neonctl CLI + Verification Script

**Rationale:**
1. CLI commands are reproducible and documentable in the setup guide — better than screenshots.
2. `neonctl` is needed anyway for T-021-02 (CI branching uses `neonctl branches create`).
3. The verification script (`scripts/verify-neon.sh`) has ongoing value: T-021-03 can reuse it, and it serves as a smoke test for the database layer independent of the full API.
4. IaC (C) is rejected — one Neon project doesn't justify a new IaC provider.

## Design Details

### Verification Script (`scripts/verify-neon.sh`)

A bash script that validates the Neon setup against all acceptance criteria:

1. **Connection check** — connect via `psql` to both direct and pooled endpoints
2. **PostGIS check** — `SELECT PostGIS_Version();`
3. **Migration check** — verify all 6 expected tables exist in `public` schema
4. **Spatial roundtrip** — insert a zone with a polygon geometry, fetch it back, verify coordinates match, delete the test data
5. **Scale-to-zero check** — report compute status (active/suspended) via Neon API or note for manual verification

The script takes two connection strings as arguments (direct + pooled) and exits non-zero on any failure.

### Connection String Updates

**Doppler (dev config):**
- `DATABASE_URL` → Neon direct endpoint (for local dev pointing at Neon)
- Note: most local dev still uses Docker Compose; Doppler config is for when you want to test against real Neon

**SSM (SST, per stage):**
- `DatabaseUrl` → Neon pooled endpoint with `sslmode=require&sslnegotiation=direct&statement_cache_size=0`

### .env.example Update

Already has Neon connection string templates. The `sslnegotiation=direct` parameter is already documented. No changes needed.

### Setup Guide Update

Update the existing `docs/active/work/T-017-02/setup-neon.md` with:
- `neonctl` commands instead of just console instructions
- Pointer to the new verification script
- Connection string for the pooled endpoint needs `sslnegotiation=direct` (currently missing from the guide's SST section)

### Cost Documentation

Document in a `docs/active/work/T-021-01/cost-analysis.md`:
- Neon Launch plan pricing: $19/mo base, includes 300 compute-hours
- Scale-to-zero: compute suspends after 5 minutes idle (configurable)
- Storage: $3.50/GiB-month (current usage negligible)
- Comparison to Railway $5/mo grandfathered plan
- Expected monthly cost for current usage pattern (low traffic, dev-only): ~$0-5/mo on free tier, or $19/mo on Launch for production features (branching, higher limits)

### What We Do NOT Change

- `crates/pt-repo/src/pool.rs` — no code changes
- `crates/plantastic-api/src/main.rs` — no code changes
- `docker-compose.yml` — local dev still uses Docker
- `infra/sst.config.ts` — infrastructure definition unchanged (secret injection pattern same)
- Migration files — unchanged, already Neon-compatible

## Rejected: Data Migration from Railway

Per ticket AC: "If starting fresh, seed data not required (migrations create schema only)." Railway had dev/test data only. Fresh Neon database with migrations applied is the correct path.

## Scenarios

This ticket is infrastructure provisioning. No scenario assertions change. The scenario dashboard should remain stable (no regressions). T-021-03 will validate Lambda → Neon under load.
