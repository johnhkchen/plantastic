# T-017-02 Structure: File Changes and Component Boundaries

## Files Modified

### `infra/sst.config.ts`
Add CORS configuration to the S3 bucket. The bucket declaration changes from bare
`new sst.aws.Bucket("Uploads")` to include a `cors` block enabling browser direct uploads.

```
Before:
  const uploads = new sst.aws.Bucket("Uploads");

After:
  const uploads = new sst.aws.Bucket("Uploads", {
    cors: {
      allowOrigins: ["*"],
      allowMethods: ["GET", "PUT", "POST"],
      allowHeaders: ["*"],
      maxAge: "1 hour",
    },
  });
```

No other SST changes. DATABASE_URL and S3_BUCKET environment wiring already exists.

### `.env.example`
Update the DATABASE_URL example to show the Neon format and document the Doppler
alternative for local development.

```
Before:
  DATABASE_URL=postgres://localhost:5432/plantastic_dev

After:
  # Local: Docker Compose Postgres or Neon direct endpoint
  DATABASE_URL=postgres://localhost:5432/plantastic_dev
  # Neon (dev): postgres://user:pass@ep-xxx.us-west-2.aws.neon.tech/plantastic_dev?sslmode=require
  # Alternative: use `doppler run` to inject secrets automatically
```

### `justfile`
Add `migrate` recipe for running database migrations.

```
migrate:
    doppler run -- sqlx migrate run --source migrations
```

Also add a `migrate-direct` recipe for when DATABASE_URL is set directly:

```
migrate-direct:
    sqlx migrate run --source migrations
```

---

## Files Created

### `.doppler.yaml`
Doppler project configuration for local development.

```yaml
setup:
  project: plantastic
  config: dev
```

This tells the Doppler CLI which project/config to use when running `doppler run`
from the project root.

### `scripts/migrate.sh`
Thin migration runner script. Checks for sqlx-cli, runs migrations against DATABASE_URL.
Supports both direct DATABASE_URL and Doppler-injected environments.

```
#!/usr/bin/env bash
set -euo pipefail
# Check sqlx-cli is installed
# Run: sqlx migrate run --source migrations
# Report: number of migrations applied
```

### `scripts/verify-deploy.sh`
Deployment verification script. Takes a Lambda Function URL as argument.
- Hits health endpoint
- Creates a test project
- Fetches it back
- Verifies response contains expected fields
- Reports cold start timing from response headers

### `docs/active/work/T-017-02/setup-neon.md`
Step-by-step guide for Neon project setup (manual steps that can't be automated):
1. Create Neon project in us-west-2
2. Enable PostGIS extension
3. Copy connection strings (pooled + direct)
4. Set SST secret: `sst secret set DatabaseUrl "..."`
5. Set Doppler secrets
6. Run migrations

---

## Files NOT Modified

### `crates/pt-repo/src/pool.rs`
No changes. SSL mode and connection timeout are connection-string-level parameters
that Neon provides in the connection URL. The pool configuration (max_connections=5,
min_connections=0, idle_timeout=30s, acquire_timeout=3s) is already correct for Lambda.

### `crates/plantastic-api/src/main.rs`
No changes. The DATABASE_URL and S3_BUCKET env var reading is already correct.

### `crates/plantastic-api/src/s3.rs`
No changes. S3 client creation uses default AWS config which works in both Lambda
(execution role) and local (env vars / credentials file).

### `worker/`
No changes. Worker deployment is a separate concern. The BACKEND_URL will be
documented in the setup guide but not configured here.

---

## Module Boundaries

This ticket touches no Rust application code. All changes are:
- **Infrastructure config** (SST) — S3 CORS
- **Developer tooling** (.doppler.yaml, justfile, scripts) — local dev experience
- **Documentation** (setup guide) — manual Neon setup steps

The boundary is clean: application code trusts DATABASE_URL and S3_BUCKET env vars.
This ticket ensures those env vars contain correct values in all environments.

---

## Ordering

1. `.doppler.yaml` + `.env.example` update (no dependencies)
2. `infra/sst.config.ts` CORS update (no dependencies)
3. `scripts/migrate.sh` (no dependencies)
4. `justfile` migrate recipes (depends on migrate.sh)
5. `scripts/verify-deploy.sh` (depends on nothing, but used last)
6. `docs/active/work/T-017-02/setup-neon.md` (references all above)
7. Milestone claim in `tests/scenarios/src/progress.rs`
