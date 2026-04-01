# Provisioning Log: T-021-01 Neon PostgreSQL

## Prerequisites

Install required tools:

```bash
# Neon CLI
brew install neonctl
# or: npm i -g neonctl

# PostgreSQL client (for psql)
brew install libpq
# or: brew install postgresql@17

# sqlx-cli (for migrations)
cargo install sqlx-cli --no-default-features --features postgres

# Doppler CLI
brew install dopplerhq/cli/doppler
```

Authenticate with Neon:
```bash
neonctl auth
```

## Step 1: Create Neon Project

```bash
neonctl projects create \
    --name plantastic \
    --region aws-us-west-2 \
    --pg-version 17
```

Record the output:
- **Project ID:** `<project-id>`
- **Branch:** `main` (default)
- **Direct endpoint:** `postgres://user:pass@ep-<id>.us-west-2.aws.neon.tech/neondb`
- **Pooled endpoint:** `postgres://user:pass@ep-<id>-pooler.us-west-2.aws.neon.tech/neondb`

Rename the default database from `neondb` to `plantastic`:
```bash
psql "postgres://user:pass@ep-<id>.us-west-2.aws.neon.tech/neondb?sslmode=require" \
    -c "ALTER DATABASE neondb RENAME TO plantastic;"
```

Update connection strings to use `/plantastic` instead of `/neondb`.

## Step 2: Enable PostGIS

PostGIS is enabled by migration 001, but verify availability first:

```bash
psql "postgres://user:pass@ep-<id>.us-west-2.aws.neon.tech/plantastic?sslmode=require" \
    -c "CREATE EXTENSION IF NOT EXISTS postgis; SELECT PostGIS_Version();"
```

Expected: PostGIS 3.5.x (on Postgres 17).

## Step 3: Run Migrations

Use the direct (non-pooled) endpoint for DDL:

```bash
DATABASE_URL="postgres://user:pass@ep-<id>.us-west-2.aws.neon.tech/plantastic?sslmode=require&sslnegotiation=direct" \
    just migrate-direct
```

Verify all 6 tables:
```bash
psql "postgres://user:pass@ep-<id>.us-west-2.aws.neon.tech/plantastic?sslmode=require" \
    -c "SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename;"
```

Expected: materials, plants, projects, tenants, tier_assignments, zones (plus the sqlx _sqlx_migrations table).

## Step 4: Run Verification Script

```bash
./scripts/verify-neon.sh \
    "postgres://user:pass@ep-<id>.us-west-2.aws.neon.tech/plantastic?sslmode=require&sslnegotiation=direct" \
    "postgres://user:pass@ep-<id>-pooler.us-west-2.aws.neon.tech/plantastic?sslmode=require&sslnegotiation=direct&statement_cache_size=0"
```

All 6 checks should pass:
- Direct endpoint connectivity
- PostGIS extension enabled
- All 6 migration tables exist
- Spatial query roundtrip
- Pooled endpoint connectivity
- Pooled endpoint query works

## Step 5: Configure Doppler (Local Dev)

```bash
doppler secrets set DATABASE_URL \
    "postgres://user:pass@ep-<id>.us-west-2.aws.neon.tech/plantastic?sslmode=require&sslnegotiation=direct" \
    -p plantastic -c dev

# Verify
doppler run -p plantastic -c dev -- env | grep DATABASE_URL
```

## Step 6: Configure SSM (Lambda via SST)

Use the **pooled** endpoint with Lambda-specific parameters:

```bash
cd infra

npx sst secret set DatabaseUrl \
    "postgres://user:pass@ep-<id>-pooler.us-west-2.aws.neon.tech/plantastic?sslmode=require&sslnegotiation=direct&statement_cache_size=0" \
    --stage dev
```

## Step 7: Store Neon API Token (for T-021-02 CI branching)

Generate an API token in Neon Console → Account → API Keys.

Store as GitHub Actions secret:
```bash
gh secret set NEON_API_KEY --body "<token>"
```

Also store the project ID for branch operations:
```bash
gh secret set NEON_PROJECT_ID --body "<project-id>"
```

## Step 8: Verify Scale-to-Zero

1. Wait 5+ minutes with no connections
2. Check Neon Console → project → compute status shows "Suspended"
3. Run verification script again — first check will trigger wake
4. Note wake time in output

## Results

> **Fill in after executing the above steps.**
>
> - [ ] Project created in us-west-2
> - [ ] PostgreSQL version: 17
> - [ ] PostGIS version: 3.5.x
> - [ ] All 6 migrations applied
> - [ ] Verification script: 6/6 checks passed
> - [ ] Doppler configured
> - [ ] SSM secret set
> - [ ] Scale-to-zero verified
> - [ ] Neon API token stored for CI
