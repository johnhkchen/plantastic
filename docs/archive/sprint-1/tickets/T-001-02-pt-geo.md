---
id: T-001-02
story: S-001
title: pt-geo-crate
type: task
status: open
priority: critical
phase: done
depends_on: [T-001-01]
---

## Context

pt-geo is the geometry foundation. Every spatial operation in the system flows through this crate — area for quotes, perimeter for edging, volume for fill materials, boolean ops for zone editing, simplification for display. Thin wrapper around the `geo` crate. Pure functions, no I/O.

## Acceptance Criteria

- Polygon area computation (sq ft) with tests against known polygons
- Perimeter computation (linear ft)
- Volume from area × depth (cu ft → cu yd conversion)
- Boolean operations: union, difference (subtract a patio from a lawn zone)
- Polygon simplification (Ramer-Douglas-Peucker) for display optimization
- All functions are pure — no I/O, no side effects
- Unit tests covering regular polygons, irregular shapes, edge cases (self-intersecting, zero-area)
- Documentation on public API
