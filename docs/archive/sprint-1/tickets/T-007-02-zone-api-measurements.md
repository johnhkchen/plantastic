---
id: T-007-02
story: S-007
title: zone-api-measurements
type: task
status: open
priority: high
phase: done
depends_on: [T-004-02, T-007-01]
---

## Context

Connect the polygon drawing component to the zone CRUD API routes (built in T-004-02). When a zone is drawn or edited, it persists to the database. The API returns computed measurements (area, perimeter) from pt-geo so the frontend can display them without duplicating geometry math in JS.

## Acceptance Criteria

- Zone save/update calls POST/PATCH /projects/:id/zones
- Zone delete calls DELETE /projects/:id/zones/:zid
- API response includes computed area_sqft and perimeter_ft for each zone
- Frontend displays area and perimeter next to each zone
- Zones reload correctly when the project page is refreshed
- S.2.1 scenario test registered and passing at ★★ (zone drawing + measurement via API)
- Claim milestone: "Zone editor: polygon drawing on plan view" in progress.rs
