---
id: T-004-02
story: S-004
title: crud-routes
type: task
status: open
priority: high
phase: done
depends_on: [T-004-01]
---

## Context

Wire the repository layer to HTTP. These are the bread-and-butter routes that the frontend needs to create and manage projects, zones, and materials. Each route validates input, calls the repository, and returns JSON. Tenant scoping is enforced — a request can only access its own tenant's data.

## Acceptance Criteria

### Project routes
- POST /projects — create project (address, client_name, client_email), returns project with id
- GET /projects — list projects for tenant
- GET /projects/:id — get project with zones
- DELETE /projects/:id — soft delete or hard delete

### Zone routes
- GET /projects/:id/zones — list zones for project
- PUT /projects/:id/zones — bulk update (from 2D editor, replaces all zones)
- POST /projects/:id/zones — add single zone
- PATCH /projects/:id/zones/:zid — update zone geometry or metadata
- DELETE /projects/:id/zones/:zid — remove zone

### Material routes
- GET /materials — list tenant's catalog
- POST /materials — create material
- PATCH /materials/:id — update material
- DELETE /materials/:id — remove material

### Tier routes
- GET /projects/:id/tiers — get all three tiers with assignments
- PUT /projects/:id/tiers/:tier — set material assignments for a tier

### All routes
- Request validation with meaningful error messages
- Tenant scoping enforced (placeholder: tenant_id from header or hardcoded for V1)
- Integration tests hitting the real API with a test database
