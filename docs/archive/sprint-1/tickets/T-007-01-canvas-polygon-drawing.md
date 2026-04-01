---
id: T-007-01
story: S-007
title: canvas-polygon-drawing
type: task
status: open
priority: high
phase: done
depends_on: []
---

## Context

The zone editor starts with a polygon drawing tool. This is a frontend-only component — no backend dependency. Draws polygons on a canvas over a placeholder background (solid color or grid; real plan view comes later with scan processing).

## Acceptance Criteria

- Svelte component: click to place polygon vertices, double-click or close to finish
- Zone type selector (bed, patio, path, lawn, wall, edging)
- Label text input
- Visual feedback: vertex handles, edge preview while drawing, fill color by zone type
- Edit mode: drag vertices to reshape, delete zone
- Multiple zones on one canvas
- Emits zone data (geometry as coordinate array, type, label) to parent component
- No API calls — this is a pure UI component. API wiring is T-007-02.
