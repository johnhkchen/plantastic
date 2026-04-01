# Structure: T-021-01 Neon Provisioning

## Files Created

### `scripts/verify-neon.sh`

Bash script that validates a Neon database setup against all T-021-01 acceptance criteria.

**Inputs:** Two positional arguments — `$1` = direct connection string, `$2` = pooled connection string.

**Sections:**
1. Argument validation — require both connection strings
2. Direct endpoint connection — `psql` connectivity check
3. PostGIS verification — `SELECT PostGIS_Version();`
4. Table verification — query `pg_tables` for all 6 expected tables
5. Spatial roundtrip — insert test zone with polygon, fetch back, verify geometry, delete
6. Pooled endpoint connection — `psql` connectivity check via `-pooler` endpoint
7. Pooled endpoint query — run a simple query to verify PgBouncer routing works
8. Summary — report pass/fail for each check

**Exit code:** 0 if all checks pass, 1 on any failure.

**Dependencies:** `psql` (standard PostgreSQL client).

### `docs/active/work/T-021-01/cost-analysis.md`

Neon cost documentation per ticket AC.

**Sections:**
- Neon pricing tiers (Free, Launch, Scale)
- Expected monthly cost for Plantastic usage pattern
- Comparison to Railway $5/mo baseline
- Scale-to-zero behavior and implications
- Recommendations

### `docs/active/work/T-021-01/provisioning-log.md`

Step-by-step log of the actual provisioning commands run and their output. Serves as both documentation and reproducibility artifact.

**Sections:**
- Prerequisites (tools installed)
- Project creation (`neonctl` commands)
- PostGIS enablement
- Migration application
- Doppler configuration
- SSM secret configuration
- Verification results

## Files Modified

### `docs/active/work/T-017-02/setup-neon.md`

Update the existing setup guide:
- Add `neonctl` CLI installation instructions
- Add `neonctl` commands alongside console instructions
- Fix pooled connection string in SST section: add `sslnegotiation=direct`
- Add pointer to `scripts/verify-neon.sh`
- Update database name from `neondb` (Neon default) to `plantastic` if we create a custom database

### `justfile`

Add recipe:
- `verify-neon` — runs `scripts/verify-neon.sh` with connection strings from Doppler

## Files NOT Modified

| File | Reason |
|---|---|
| `crates/pt-repo/src/pool.rs` | Already Neon-ready (T-020-02) |
| `crates/plantastic-api/src/main.rs` | Reads DATABASE_URL, no changes |
| `.env.example` | Already has Neon templates |
| `docker-compose.yml` | Local dev unchanged |
| `infra/sst.config.ts` | Secret injection pattern unchanged |
| `migrations/*.sql` | Already Neon-compatible |
| `.doppler.yaml` | Project/config already correct |

## Module Boundaries

This ticket produces no Rust code. All deliverables are:
- Shell scripts (verification tooling)
- Documentation (cost analysis, provisioning log)
- Configuration (Doppler secrets, SSM secrets — set externally, not in code)

## Ordering

1. Write `scripts/verify-neon.sh` first — it's the acceptance test
2. Write `cost-analysis.md` — can be done independently
3. Provision Neon (documented in `provisioning-log.md`) — requires the script to exist for verification
4. Update `setup-neon.md` — after provisioning confirms actual values
5. Add Justfile recipe — after script exists
