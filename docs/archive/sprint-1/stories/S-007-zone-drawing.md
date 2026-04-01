---
id: S-007
epic: E-003
title: Zone Drawing
status: open
priority: high
dependencies:
  - S-004
---

# S-007: Zone Drawing

## Purpose

The zone editor is how landscapers and clients define what goes where. Without it, there's no geometry to quote against and no design to visualize. This story delivers the polygon drawing component and wires it to the API for persistence.

## Scope

- Canvas-based polygon drawing tool over a placeholder plan view (actual scan integration comes later)
- Zone type assignment (bed, patio, path, lawn, wall, edging)
- Zone label input
- Live area/perimeter/volume display computed from pt-geo via the API
- Zone CRUD through the API (create, update geometry, delete)
- Targets S.2.1 scenario at ★★

## Tickets

- T-007-01: Canvas polygon drawing component
- T-007-02: Zone API persistence + live measurements
