---
id: T-008-01
story: S-008
title: quote-api-route
type: task
status: open
priority: high
phase: done
depends_on: [T-004-02]
---

## Context

Wire pt-quote to an HTTP endpoint. The computation engine already works (S.3.1/S.3.2 passing at ★☆☆☆☆). This ticket makes it reachable over the network — loads zones from the database, fetches material assignments for the requested tier, calls compute_quote, returns JSON.

## Acceptance Criteria

- GET /projects/:id/quote/:tier returns Quote JSON (line items, subtotal, total)
- Loads zone geometry from PostGIS, tier assignments, and materials from the tenant catalog
- Returns 404 for missing project, 400 for invalid tier name
- Returns empty quote (no line items, $0 total) if tier has no assignments yet
- Integration test: create project with zones + assignments via API, fetch quote, verify correct totals
