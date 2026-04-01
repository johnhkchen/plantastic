---
id: T-003-02
story: S-003
title: sqlx-repository-layer
type: task
status: open
priority: critical
phase: done
depends_on: [T-003-01]
---

## Context

The repository layer bridges domain types and the database. It provides typed Rust functions for CRUD operations, mapping between pt-project/pt-materials types and database rows via sqlx. This layer lives in the API crate but is designed as a module that could be extracted if needed.

## Acceptance Criteria

- sqlx with Postgres runtime and compile-time query checking
- Connection pool configured for Lambda (max_connections=5, min_connections=0, idle_timeout short)
- Project repository: create, get_by_id, list_by_tenant, update_status, delete
- Zone repository: list_by_project, bulk_upsert, add, update, delete
- Material repository: list_by_tenant, create, update, delete
- TierAssignment repository: get_by_project_and_tier, set_assignments (bulk upsert)
- GeoJSON ↔ PostGIS geometry conversion for zone polygons
- Integration tests against a real Postgres database (not mocks)
- Round-trip test: create project with zones → retrieve → verify geometry matches
