# T-017-02 Research: Railway→Neon PostGIS + S3 + Secrets Wiring

## Ticket Context

Connect deployed Lambda (from T-017-01) to managed PostgreSQL with PostGIS, wire S3
bucket, and establish secrets management via Doppler (dev) and SSM (prod).

**Critical update:** Memory records (2026-03-31) indicate Railway PostgreSQL is being
replaced by Neon (neon.tech). This research reflects the Neon decision. The ticket
title references Railway but the implementation target is Neon.

---

## 1. Current Infrastructure State (from T-017-01)

### SST Configuration (`infra/sst.config.ts`)
- Lambda function: `provided.al2023`, arm64, 256MB, 30s timeout
- S3 bucket: `sst.aws.Bucket("Uploads")` — already provisioned
- Secret: `sst.Secret("DatabaseUrl")` — placeholder, reads from SSM
- Environment wiring: `DATABASE_URL`, `S3_BUCKET`, `RUST_LOG` already passed to Lambda
- Region: `us-west-2`
- Function URL with `RESPONSE_STREAM` mode (SSE support)

### Lambda Binary (`crates/plantastic-api/src/main.rs`)
- Reads `DATABASE_URL` from env, falls back to `localhost:5432/plantastic_dev`
- Reads `S3_BUCKET` from env, falls back to `plantastic-dev`
- Auto-detects Lambda vs local via `AWS_LAMBDA_RUNTIME_API`
- Creates pool via `pt_repo::create_pool()`, S3 client via `aws_config`

### Connection Pool (`crates/pt-repo/src/pool.rs`)
- `max_connections=5`, `min_connections=0`
- `idle_timeout=30s`, `acquire_timeout=3s`
- Tuned for Lambda freeze/thaw cycles
- **No SSL mode configured** — critical for Neon (requires `sslmode=require`)
- **No connection string parameters** — Neon pooler needs specific settings

### S3 Integration (`crates/plantastic-api/src/s3.rs`)
- `create_s3_client()` uses default AWS config (execution role in Lambda)
- `upload_bytes()`, `download_bytes()`, `presigned_get_url()` all implemented
- **No CORS configuration** on the bucket itself (needed for browser uploads)

---

## 2. Database: Neon vs Railway

### Why Neon (from memory records)
- Railway default Postgres template doesn't reliably support PostGIS
- Neon: PostGIS as first-class extension, copy-on-write branching, built-in PgBouncer
- Neon has `aws-us-west-2` region matching Lambda (low latency)
- Railway: grandfathered $5/mo plan, no connection pooling, no branching

### Neon Specifics
- Connection string format: `postgres://user:pass@ep-xxx.us-west-2.aws.neon.tech/dbname`
- Pooled endpoint: `postgres://user:pass@ep-xxx-pooler.us-west-2.aws.neon.tech/dbname`
- **Must use pooled endpoint for Lambda** (PgBouncer, handles connection storms)
- PostGIS enabled via `CREATE EXTENSION IF NOT EXISTS postgis;` (already in migration 001)
- **Known risk:** sqlx cold-start hang with Neon — mitigate with `connect_timeout` param
  in connection string and retry logic

### Connection String Considerations
- Neon requires SSL: append `?sslmode=require` to connection string
- Pooled connections: use `-pooler` endpoint suffix
- Lambda cold start: `connect_timeout=5` parameter recommended
- Full string: `postgres://user:pass@ep-xxx-pooler.us-west-2.aws.neon.tech/dbname?sslmode=require&connect_timeout=5`

---

## 3. Migrations

Six migrations in `migrations/` directory:
1. `001-create-tenants.sql` — tenants + `CREATE EXTENSION IF NOT EXISTS postgis`
2. `002-create-projects.sql` — projects with `GEOGRAPHY(POINT, 4326)`
3. `003-create-zones.sql` — zones with `GEOMETRY(POLYGON, 4326)` + GIST index
4. `004-create-materials.sql` — material catalog
5. `005-create-tier-assignments.sql` — tier assignments
6. `006-create-plants.sql` — plant database

All migrations have corresponding `.down.sql` files for rollback.
PostGIS extension created in migration 001 — must be available on the target database.
No migration runner configured yet (sqlx-cli assumed for manual application).

---

## 4. Secrets Management

### Current State
- `.env.example` defines: `DATABASE_URL`, `PORT`, `S3_BUCKET`, `AWS_REGION`, AWS creds
- `dotenvy::dotenv().ok()` loads `.env` in local mode
- SST config uses `sst.Secret("DatabaseUrl")` which reads from SSM Parameter Store
- **No Doppler configuration** exists (no `.doppler.yaml`, no project setup)

### Doppler (dev)
- CLI tool for secrets management
- `.doppler.yaml` in project root configures project/config
- `doppler run -- cargo run -p plantastic-api` injects secrets as env vars
- Integrates with SSM via Doppler's AWS integration (can sync to SSM)

### SSM Parameter Store (prod)
- SST `sst.Secret` creates SSM parameters at `/sst/{app}/{stage}/Secret/{name}/value`
- Set via `sst secret set DatabaseUrl "postgres://..."` per stage
- Lambda execution role automatically gets read access via SST linking
- No additional IAM configuration needed for SST-managed secrets

---

## 5. S3 Bucket Configuration

### Current State
- Bucket created by SST, linked to Lambda (IAM permissions automatic)
- No CORS policy configured
- Scan upload flow: multipart → S3 `scans/{project_id}/raw.ply`
- Generated artifacts: glTF mesh, PNG plan view stored in S3

### CORS Requirements (for browser direct upload)
- Allow origins: `*` initially (lock down to domain later)
- Allow methods: PUT, POST, GET
- Allow headers: Content-Type, x-amz-content-sha256, etc.
- Max age: 3600s

### SST CORS Configuration
- SST v3 `sst.aws.Bucket` supports `cors` property
- Can specify allowed origins, methods, headers directly in config

---

## 6. Worker Proxy (`worker/`)

### Current State (`worker/wrangler.toml`, `worker/src/index.ts`)
- Cloudflare Worker: `plantastic-api-proxy`
- Proxies requests to Lambda Function URL
- CORS handling, rate limiting (60/min IP, 200/session)
- Needs `BACKEND_URL` env var set to Lambda Function URL
- **Not yet deployed** — T-017-02 should document the connection

---

## 7. Verification Strategy

### Cold Start Testing
- Lambda cold start + Neon connection: measure total time
- Expected: ~500ms for Neon pooled connection from us-west-2 Lambda
- Neon cold-start hang risk: connect_timeout mitigates this

### End-to-End
- Create project via API → stored in Neon → fetch back
- Spatial query: verify PostGIS works (zone creation with geometry)
- S3: upload test file → verify storage

### Connection Pooling
- Neon pooler handles connection storms from Lambda scale-out
- `max_connections=5` per Lambda instance is appropriate
- Monitor: Neon dashboard shows active connections

---

## 8. Files That Will Be Modified

- `infra/sst.config.ts` — CORS on S3 bucket, possibly connection timeout env vars
- `.env.example` — update with Neon connection string format
- `crates/pt-repo/src/pool.rs` — potentially add SSL/timeout configuration
- New: `.doppler.yaml` — Doppler project configuration
- New: `scripts/migrate.sh` — migration runner script
- New: `docs/active/work/T-017-02/` — all phase artifacts

---

## 9. Risks and Constraints

1. **Neon cold-start hang** — sqlx/tokio-postgres can hang connecting to Neon on
   Lambda cold start. Mitigation: `connect_timeout` in connection string, retry logic.
2. **SSL requirement** — Neon requires SSL; current pool.rs doesn't configure SSL mode.
   Connection string parameter `sslmode=require` should suffice for sqlx.
3. **Migration runner** — No automated migration runner. Manual `sqlx migrate run`
   against Neon. Consider adding a `just migrate` recipe.
4. **Doppler availability** — Doppler CLI must be installed locally. Not a blocker
   for prod (SSM is independent).
5. **Railway ticket title** — Ticket title says "railway" but we're targeting Neon.
   Implementation follows the latest decision; ticket title is cosmetic.
