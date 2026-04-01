---
id: T-003-01
story: S-003
title: postgis-migrations
type: task
status: open
priority: critical
phase: done
depends_on: [T-002-02]
---

## Context

The database schema translates the domain models into persistent storage. PostGIS gives us spatial queries on zone geometry and native GeoJSON support. Migrations are versioned SQL files applied in order. The schema must support multi-tenancy from day one — every project and material is scoped to a tenant.

## Acceptance Criteria

- Migration 001: tenants table (id UUID, name, logo_url, brand_color, contact JSONB, timestamps)
- Migration 002: projects table (id UUID, tenant_id FK, client_name, client_email, address, location GEOGRAPHY(POINT), scan_ref JSONB, baseline JSONB, status, timestamps)
- Migration 003: zones table (id UUID, project_id FK CASCADE, geometry GEOMETRY(POLYGON, 4326), zone_type, label, sort_order)
- Migration 004: materials table (id UUID, tenant_id FK, name, category, unit, price_per_unit DECIMAL, depth_inches, extrusion JSONB, texture_key, photo_key, supplier_sku)
- Migration 005: tier_assignments table (id UUID, project_id FK CASCADE, tier, zone_id FK CASCADE, material_id FK, overrides JSONB, UNIQUE on project+tier+zone)
- Migration 006: plants table (id UUID, common_name, botanical_name, sun/climate/size fields, tags array, photo_url) — platform-level, no tenant FK
- PostGIS extension enabled
- Spatial index on zones.geometry
- All migrations apply cleanly on a fresh Postgres 16 + PostGIS 3.4 database
- Down migrations for rollback
