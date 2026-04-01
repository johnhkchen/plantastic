---
id: S-021
epic: E-009
title: Neon Migration & CI
status: open
priority: high
dependencies:
  - S-020
---

# S-021: Neon Migration & CI

## Purpose

Replace Railway with Neon as the managed PostgreSQL provider. Neon gives us PostGIS as a first-class extension, database branching for CI, built-in PgBouncer, and same-region deployment with Lambda (us-west-2). Database branching means CI integration tests get a real Postgres with production schema in <1 second, no Docker needed in GitHub Actions.

## Scope

- Provision Neon project (PG 17, us-west-2, PostGIS enabled)
- Apply all migrations, verify spatial queries work
- CI workflow: create ephemeral Neon branch → run integration tests → delete branch
- Update E-008 deployment epic to reference Neon instead of Railway
- Update SST config / secrets to use Neon connection strings
- Validate Lambda → Neon connection (cold start timing, pooler behavior)
- Update T-017-02 acceptance criteria (Neon replaces Railway)

## Tickets

- T-021-01: Neon provisioning + migration
- T-021-02: CI integration tests via Neon branching
- T-021-03: Lambda connection validation + E-008 updates
