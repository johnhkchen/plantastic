# Research: T-021-01 Neon Provisioning

## Ticket Summary

Replace Railway PostgreSQL with Neon as the managed database provider. Provision a Neon project in us-west-2 with PostGIS, apply all 6 migrations, store connection strings in Doppler (local dev) and SSM (Lambda), and verify spatial queries work end-to-end.

## Current State

### Database Infrastructure

**Local development** uses Docker Compose (`docker-compose.yml`) with `postgis/postgis:18-3.6`. Migrations are auto-applied on container startup via `scripts/docker-init-db.sh` mounted into the Postgres entrypoint. Connection: `postgres://plantastic:plantastic@localhost:5432/plantastic`.

**Production** uses SST v3 (`infra/sst.config.ts`). The Lambda function receives `DATABASE_URL` from an SSM Secret (`sst.Secret("DatabaseUrl")`). The Lambda binary auto-detects Lambda mode via `AWS_LAMBDA_RUNTIME_API`.

**Railway** was the previous provider (T-017-02). No active Railway configuration remains in the codebase — the ticket is done and all references point forward to Neon.

### Connection Handling (T-020-02 — complete)

`crates/pt-repo/src/pool.rs` provides `PoolConfig` with Lambda-tuned defaults:
- `connect_timeout: 15s` (covers Neon cold-start of 3-8s)
- `max_connections: 5`, `min_connections: 0` (scale-to-zero)
- `max_retries: 3`, `initial_backoff: 500ms` (exponential backoff)
- `is_transient()` classifies Io/Tls/PoolTimedOut as retryable

sqlx 0.8 natively handles all Neon-specific connection string parameters:
- `sslmode=require` — required for Neon
- `sslnegotiation=direct` — skips SSLRequest round-trip
- `statement_cache_size=0` — required for PgBouncer transaction mode (pooled endpoint)

No application code changes are needed for Neon. The `-pooler` hostname suffix is pure DNS routing.

### Migrations (6 total)

| # | File | PostGIS Usage |
|---|---|---|
| 001 | create-tenants | `CREATE EXTENSION IF NOT EXISTS postgis;` |
| 002 | create-projects | `GEOGRAPHY(POINT, 4326)` for location |
| 003 | create-zones | `GEOMETRY(POLYGON, 4326)` + GiST index |
| 004 | create-materials | None (catalog table) |
| 005 | create-tier-assignments | None (join table) |
| 006 | create-plants | None (platform plants) |

PostGIS is a first-class Neon extension — migration 001's `CREATE EXTENSION` will succeed without manual setup on Postgres 14+.

### Secrets Management

**Doppler** (`.doppler.yaml`): project `plantastic`, config `dev`. Used for local dev via `doppler run`. Justfile recipe `just migrate` uses Doppler injection.

**SSM** (SST): `sst.Secret("DatabaseUrl")` set per stage via `npx sst secret set DatabaseUrl "<url>" --stage dev`.

### Existing Setup Guide

`docs/active/work/T-017-02/setup-neon.md` (135 lines) contains a complete walkthrough: create project, enable PostGIS, run migrations, configure Doppler, set SST secrets, deploy & verify. This guide was written during T-017-02 and covers the exact provisioning steps this ticket requires.

### Downstream Dependencies

- **T-021-02** (CI Neon branching): needs the Neon project + API token to create ephemeral branches
- **T-021-03** (Lambda connection validation): needs deployed Lambda connected to Neon pooled endpoint

### What This Ticket Does NOT Cover

- CI branching workflow (T-021-02)
- Cold-start benchmarking (T-021-03)
- Data migration from Railway (ticket says: if starting fresh, no seed needed — migrations create schema only)
- Application code changes (none required — T-020-02 already handles Neon)

## Key Files

| Path | Role |
|---|---|
| `crates/pt-repo/src/pool.rs` | Connection pool with retry (no changes needed) |
| `crates/plantastic-api/src/main.rs` | Reads `DATABASE_URL`, creates pool |
| `.env.example` | Already has Neon direct + pooled templates |
| `.doppler.yaml` | Doppler project config |
| `infra/sst.config.ts` | SST Lambda + SSM secret |
| `migrations/*.sql` | 6 up + 6 down migrations |
| `scripts/migrate.sh` | Migration runner (sqlx-cli) |
| `docs/active/work/T-017-02/setup-neon.md` | Existing Neon setup guide |

## Constraints

1. Neon project must be in **us-west-2** to co-locate with Lambda
2. PostgreSQL 17 required for stable PostGIS 3.5
3. Pooled endpoint required for Lambda; direct endpoint for migrations
4. Connection strings must include `sslmode=require` and `sslnegotiation=direct`
5. No application code changes should be needed — the connection layer is already Neon-ready
