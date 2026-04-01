---
id: T-038-02
story: S-038
title: fix-tenant-isolation
type: task
status: open
priority: critical
phase: ready
depends_on: []
---

## Context

S.INFRA.2 (Tenant isolation) fails at "POST /projects/:id/zones as Tenant A: expected 201, got 422 Unprocessable Entity." The zone creation payload is rejected by the API's validation. The API works correctly — the test sends the wrong format.

## Acceptance Criteria

- Diagnose: what does the 422 response body say? (add error body logging to the scenario)
- Fix the zone POST payload to match what the API expects:
  - GeoJSON geometry must be a proper GeoJSON Polygon object
  - The zone POST endpoint likely expects `{ "label": "...", "zone_type": "...", "geometry": { "type": "Polygon", "coordinates": [...] } }`
  - Compare with the working S.INFRA.1 zone POST (which passes at the same API)
- S.INFRA.2 passes at ★★☆☆☆ with DATABASE_URL
- `just check` (no DATABASE_URL) still passes

## Implementation Notes

- S.INFRA.1 already creates zones successfully — copy the exact payload format
- The most likely issue: S.INFRA.2 test was written independently and used a different JSON shape
- Add the 422 response body to the error message so future failures are self-diagnosing:
  `format!("POST /zones: expected 201, got {status}: {body}")`
