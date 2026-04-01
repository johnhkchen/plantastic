---
id: T-021-01
story: S-021
title: neon-provisioning
type: task
status: open
priority: high
phase: done
depends_on: [T-020-02]
---

## Context

Provision Neon as the managed PostgreSQL provider, replacing Railway. Neon provides PostGIS as a first-class extension, database branching for CI, built-in PgBouncer, and can be co-located with Lambda in us-west-2.

## Acceptance Criteria

### Neon project
- Project created in `aws-us-west-2` region
- PostgreSQL 17 (stable PostGIS 3.5 support)
- PostGIS extension enabled: `CREATE EXTENSION IF NOT EXISTS postgis;`
- All 6 migrations applied successfully
- Verify: spatial queries work (insert a zone with geometry, fetch it back)

### Connection strings
- Production connection string (direct): stored in Doppler / SSM
- Pooled connection string (`-pooler` suffix): used by Lambda
- Connection validated from local machine (`psql`) and from a test Lambda invocation

### Data
- If Railway has existing data, document migration path (pg_dump/pg_restore)
- If starting fresh, seed data not required (migrations create schema only)

### Cost verification
- Document expected monthly cost on Launch plan for current usage pattern
- Verify scale-to-zero works (compute suspends after idle period)
- Compare to Railway $5/mo baseline
