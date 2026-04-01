---
id: T-017-02
story: S-017
title: railway-s3-secrets
type: task
status: open
priority: high
phase: done
depends_on: [T-017-01]
---

## Context

Connect the deployed Lambda to Neon PostGIS and provision S3 for object storage. Wire secrets through Doppler (dev) and SSM (prod) so the Lambda can reach both.

> **Note:** Originally scoped for Railway. Migrated to Neon (T-021-01) for PostGIS support, database branching, built-in PgBouncer, and us-west-2 co-location with Lambda.

## Acceptance Criteria

### Neon PostGIS
- PostGIS extension enabled on Neon project (us-west-2)
- All 6 migrations applied cleanly (up scripts)
- Direct and pooled connection strings stored in Doppler/SSM
- Verify: connect from local machine, run a spatial query (`just verify-neon`)

### S3
- SST creates an S3 bucket for scan artifacts and generated files
- Bucket name available to Lambda via environment variable
- CORS configured for direct browser uploads (future: scan upload)

### Secrets
- Doppler project "plantastic" with dev config
- DATABASE_URL, S3 bucket name, and any API keys in Doppler
- `doppler run` works for local development
- SSM parameters created for prod: Lambda reads DATABASE_URL from SSM at startup
- SST wires SSM parameter ARNs to Lambda's environment

### Verification
- Lambda connects to Neon PostGIS on cold start via pooled endpoint (measure connection time)
- Create a project via the deployed API → stored in Neon → fetch it back
- Connection pooling handled by Neon's built-in PgBouncer (`-pooler` endpoint suffix)
- Connection string tuning: `sslnegotiation=direct`, `statement_cache_size=0` (see T-020-02)
