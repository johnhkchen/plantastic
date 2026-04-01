---
id: T-025-02
story: S-025
title: full-stack-round-trip-scenario
type: task
status: open
priority: high
phase: ready
depends_on: [T-025-01]
---

## Context

S.INFRA.1 (Full-stack round-trip) is NotImplemented but 5/8 prereqs are delivered. The scenario validates the complete CRUD lifecycle through the API: create project → add zones → add materials → assign to tier → compute quote → delete → verify gone.

## Acceptance Criteria

- S.INFRA.1 scenario test implemented in `tests/scenarios/src/suites/infrastructure.rs`
- Test flow (requires DATABASE_URL):
  1. POST /projects → 201, capture project_id
  2. GET /projects/:id → 200, verify data matches
  3. POST /projects/:id/zones (12×15 ft patio geometry) → 201
  4. GET /projects/:id/zones → zone present with correct geometry
  5. POST /materials (Travertine Pavers, $8.50/sqft) → 201
  6. PUT /projects/:id/tiers/good (assign material to zone) → 200
  7. GET /projects/:id/quote/good → 200, verify line items
     - Expected: 12×15=180 sqft × $8.50 = $1,530.00 (computed independently)
  8. DELETE /projects/:id → 200
  9. GET /projects/:id → 404
- Fallback: if no DATABASE_URL, return `Blocked("no DATABASE_URL")`
- Pass at ★★☆☆☆ integration (full API round-trip)
- Claim milestones in `progress.rs`:
  - "SvelteKit frontend + CF Worker proxy" — note: frontend exists, CF Worker deployed
  - "pt-project: Project/Zone/Tier model + GeoJSON serde" — note: model exists in pt-repo
  - "pt-quote: quantity takeoff engine" — note: pt-quote crate computes quotes, API serves them
- `just check` passes

## Implementation Notes

- Reuse `api_helpers` module from quoting/infrastructure suites
- Quote verification: the expected total ($1,530.00) must be computed in the test as `12 × 15 × 8.50 = 1530.00`, not by calling pt-geo or pt-quote
- GeoJSON geometry for the 12×15 patio: `{"type":"Polygon","coordinates":[[[0,0],[12,0],[12,15],[0,15],[0,0]]]}`
- This test exercises the same code path as a real user creating a project and getting a quote
