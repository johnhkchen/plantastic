# Neon PostGIS Setup Guide (T-017-02)

## Prerequisites

- Neon account (neon.tech)
- `neonctl` CLI: `brew install neonctl` or `npm i -g neonctl`
- `sqlx-cli` installed: `cargo install sqlx-cli --no-default-features --features postgres`
- `psql`: `brew install libpq` or `brew install postgresql@17`
- Doppler CLI installed: `brew install dopplerhq/cli/doppler`
- SST CLI: `npx sst` from `infra/` directory

---

## 1. Create Neon Project

### Via CLI (recommended)

```bash
neonctl auth
neonctl projects create --name plantastic --region aws-us-west-2 --pg-version 17
```

### Via Console

1. Go to https://console.neon.tech
2. Create new project:
   - Name: `plantastic`
   - Region: `AWS us-west-2` (matches Lambda)
   - Postgres version: 17 (latest stable)

### Connection Strings

Note the two connection strings from the output:
   - **Direct** (for migrations, local dev): `postgres://user:pass@ep-xxx.us-west-2.aws.neon.tech/plantastic`
   - **Pooled** (for Lambda): `postgres://user:pass@ep-xxx-pooler.us-west-2.aws.neon.tech/plantastic`

## 2. Enable PostGIS

PostGIS is available as a first-class extension on Neon. Migration 001 runs:
```sql
CREATE EXTENSION IF NOT EXISTS postgis;
```

This will succeed automatically — no manual extension setup needed.

## 3. Run Migrations

Using the **direct** (non-pooled) connection string:

```bash
DATABASE_URL="postgres://user:pass@ep-xxx.us-west-2.aws.neon.tech/neondb?sslmode=require" \
    just migrate-direct
```

Verify all 6 migrations applied:
```bash
psql "postgres://user:pass@ep-xxx.us-west-2.aws.neon.tech/neondb?sslmode=require" \
    -c "SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename;"
```

Expected tables: `materials`, `plants`, `projects`, `tenants`, `tier_assignments`, `zones`

Verify PostGIS:
```bash
psql "postgres://user:pass@ep-xxx.us-west-2.aws.neon.tech/neondb?sslmode=require" \
    -c "SELECT PostGIS_Version();"
```

## 4. Configure Doppler (Local Dev)

```bash
# Login to Doppler
doppler login

# Create project (if not exists)
doppler projects create plantastic

# Set up dev config
doppler configs create dev -p plantastic

# Set secrets (use DIRECT endpoint for local dev — no pooler needed)
doppler secrets set DATABASE_URL "postgres://user:pass@ep-xxx.us-west-2.aws.neon.tech/neondb?sslmode=require" \
    -p plantastic -c dev
doppler secrets set S3_BUCKET "plantastic-dev" -p plantastic -c dev
doppler secrets set AWS_REGION "us-west-2" -p plantastic -c dev

# Verify
doppler run -- env | grep DATABASE_URL
```

## 5. Configure SST Secret (Prod)

From the `infra/` directory, set the DATABASE_URL secret using the **pooled** endpoint:

```bash
cd infra

# Set for dev stage
npx sst secret set DatabaseUrl \
    "postgres://user:pass@ep-xxx-pooler.us-west-2.aws.neon.tech/plantastic?sslmode=require&sslnegotiation=direct&statement_cache_size=0" \
    --stage dev

# Set for production stage (when ready)
npx sst secret set DatabaseUrl \
    "postgres://user:pass@ep-xxx-pooler.us-west-2.aws.neon.tech/plantastic?sslmode=require&sslnegotiation=direct&statement_cache_size=0" \
    --stage production
```

Note: Use the **pooled** (`-pooler`) endpoint for Lambda. The `sslnegotiation=direct`
parameter skips a round-trip (~50-100ms faster). The `statement_cache_size=0` is
required for PgBouncer transaction mode. The retry logic in `pt_repo::pool` (T-020-02)
handles Neon cold-start timing.

## 6. Verify Neon Setup

Before deploying, verify the database is correctly provisioned:

```bash
just verify-neon \
    "postgres://user:pass@ep-xxx.us-west-2.aws.neon.tech/plantastic?sslmode=require&sslnegotiation=direct" \
    "postgres://user:pass@ep-xxx-pooler.us-west-2.aws.neon.tech/plantastic?sslmode=require&sslnegotiation=direct&statement_cache_size=0"
```

All 6 checks should pass (connectivity, PostGIS, tables, spatial roundtrip, pooled endpoint).

## 7. Deploy and Verify

```bash
# Build Lambda binary
just build-lambda

# Deploy
cd infra && npx sst deploy --stage dev

# Note the apiUrl output, then:
./scripts/verify-deploy.sh <api-url>
```

## 7. Connection String Reference

| Environment | Endpoint | Why |
|---|---|---|
| Local dev | Direct (non-pooled) | Single connection, no pooler overhead |
| Migrations | Direct (non-pooled) | DDL statements need direct connections |
| Lambda (SST) | Pooled (`-pooler`) | Handles connection storms from Lambda scale-out |
| CI (future) | Neon branch + direct | Ephemeral branches for test isolation |

## Troubleshooting

### Cold-start hang
If Lambda hangs on first invocation (>10s), check:
- Connection string has `connect_timeout=5`
- Using pooled endpoint (not direct)
- Neon project is in us-west-2 (same region as Lambda)

### SSL errors
Neon requires SSL. Ensure `?sslmode=require` is in the connection string.

### PostGIS not found
If migration 001 fails with "extension postgis not available":
- Verify Neon project uses Postgres 16+ (PostGIS is available on 14+)
- Check Neon dashboard → Extensions tab
