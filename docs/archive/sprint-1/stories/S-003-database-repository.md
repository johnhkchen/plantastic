---
id: S-003
epic: E-001
title: Database & Repository
status: open
priority: critical
dependencies:
  - S-002
---

# S-003: Database & Repository

## Purpose

Persist and retrieve the domain models. The PostGIS schema stores projects with spatial zone geometry, tenant-scoped material catalogs, and tier assignments. The repository layer provides typed Rust functions that map between domain types and database rows.

## Scope

- PostgreSQL/PostGIS migrations: tenants, projects, zones (with GEOMETRY column), materials, tier_assignments, plants
- sqlx-based repository layer: create/read/update/delete for projects, zones, materials, tier assignments
- Spatial queries: zones within a project boundary, area computation validation against PostGIS
- Connection pool configuration for Lambda (short-lived connections)

## Tickets

- T-003-01: PostGIS migrations
- T-003-02: sqlx repository layer
