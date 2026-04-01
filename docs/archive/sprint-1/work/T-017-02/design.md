# T-017-02 Design: Neon PostGIS + S3 CORS + Secrets Wiring

## Decision Summary

Use Neon (neon.tech) for managed PostgreSQL with PostGIS in us-west-2, Doppler for
local dev secrets, SST Secrets (backed by SSM) for prod, and add CORS to the SST
S3 bucket. Keep code changes minimal — most wiring is configuration, not application code.

---

## Option Analysis

### Database: Neon vs Railway

**Option A: Railway PostgreSQL (ticket original)**
- Pros: Already provisioned, $5/mo grandfathered plan
- Cons: No built-in PostGIS template, no connection pooling, no database branching,
  not co-located with Lambda (adds latency), being retired per project decision

**Option B: Neon PostgreSQL ✓ (selected)**
- Pros: PostGIS first-class, built-in PgBouncer (pooled endpoints), us-west-2 matching
  Lambda, copy-on-write branching for CI, generous free tier
- Cons: Cold-start connection hang risk (mitigated with connect_timeout), newer service
- Rationale: Project decision already made (memory: 2026-03-31). Neon's pooler solves
  the Lambda connection storm problem that Railway can't address.

### Secrets: Doppler + SSM vs SSM-only vs Doppler-only

**Option A: Doppler for dev, SST Secrets (SSM) for prod ✓ (selected)**
- Pros: Doppler gives easy local dev experience (`doppler run`), SST Secrets are
  already configured and give Lambda automatic IAM access
- Cons: Two systems to maintain
- Rationale: SST already uses SSM via `sst.Secret()`. Doppler adds convenience for
  local dev without replacing the prod path. This is additive, not a migration.

**Option B: SSM-only (skip Doppler)**
- Pros: Single system
- Cons: Clunky for local dev (need AWS CLI + SSM lookups or manual .env management)
- Rejected: Adds friction to local development workflow

**Option C: Doppler-only (replace SST Secrets)**
- Pros: Single system
- Cons: Requires Doppler AWS integration to sync to SSM, adds complexity to SST config
- Rejected: Over-engineers the prod path; SST's built-in secret handling is simpler

### S3 CORS: SST config vs bucket policy

**Option A: SST `cors` property on bucket ✓ (selected)**
- Pros: Declarative, version-controlled, automatic on deploy
- Cons: None meaningful
- Rationale: SST v3 supports CORS directly on `sst.aws.Bucket`. Simple and correct.

**Option B: Manual bucket CORS via AWS CLI**
- Rejected: Not version-controlled, manual step, easy to forget

### Connection Pool SSL: Connection string vs sqlx config

**Option A: SSL via connection string parameter ✓ (selected)**
- Use `?sslmode=require` in the DATABASE_URL
- Pros: No code changes needed, sqlx respects connection string parameters
- Cons: Depends on operator setting the right connection string
- Rationale: Neon provides connection strings with sslmode already. No code change
  needed in pool.rs — the connection string carries the SSL requirement.

**Option B: Programmatic SSL configuration in pool.rs**
- Rejected: Over-engineering. The connection string approach works and keeps
  configuration external to code, which is the right place for it.

---

## Design Decisions

### 1. Neon Setup (manual, documented)
- Create Neon project "plantastic" in us-west-2
- Enable PostGIS extension
- Run migrations via `sqlx migrate run`
- Extract pooled connection string for Lambda, direct string for migrations
- **Not automated** — Neon project creation is a one-time manual step

### 2. SST Changes
- Add CORS configuration to the S3 bucket for browser uploads
- No changes to Lambda function config (DATABASE_URL and S3_BUCKET already wired)
- Secret value set via `sst secret set DatabaseUrl "neon-connection-string"`

### 3. Doppler Configuration
- Create `.doppler.yaml` pointing to project "plantastic", config "dev"
- Secrets: DATABASE_URL (Neon direct endpoint for local dev), S3_BUCKET, AWS_REGION
- `doppler run -- cargo run -p plantastic-api` for local development
- **Doppler project setup is manual** — `.doppler.yaml` just points to it

### 4. Migration Script
- `scripts/migrate.sh` — thin wrapper around `sqlx migrate run`
- Takes DATABASE_URL from env or Doppler
- `just migrate` recipe in justfile
- Runs all 6 migrations against target database

### 5. Connection Pool — No Code Changes
- pool.rs remains as-is
- SSL, timeout, and pooling are connection-string-level concerns
- Neon pooled string: `postgres://user:pass@ep-xxx-pooler.us-west-2.aws.neon.tech/dbname?sslmode=require`
- `connect_timeout=5` added to connection string for cold-start protection
- `acquire_timeout=3s` in pool.rs is the app-level safety net

### 6. Verification Script
- `scripts/verify-deploy.sh` — hit the deployed Lambda Function URL with curl
- Health check, create project, fetch project, verify spatial query
- Documents cold start time from logs

---

## What This Ticket Does NOT Do

- Does not migrate data from Railway to Neon (no data exists yet)
- Does not set up CI database branching (future ticket)
- Does not deploy the Cloudflare Worker (separate concern)
- Does not implement presigned upload URLs for browser direct upload (S3 CORS enables it)
- Does not add retry logic for Neon cold-start hangs (connect_timeout is sufficient
  for now; retry is a future hardening ticket if needed)

---

## Scenario Impact

This ticket is infrastructure — no scenario directly flips to green. It unblocks:
- S.1.x (scan pipeline): S3 bucket accessible from Lambda
- S.3.x (quoting): database accessible from Lambda
- All deployed API scenarios: Lambda can reach Neon and S3

Milestone to claim: "Neon PostGIS + S3 + secrets wired for Lambda deployment"
