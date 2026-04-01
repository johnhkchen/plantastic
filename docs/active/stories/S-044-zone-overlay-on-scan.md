---
id: S-044
epic: E-017
title: Zone Overlay on Scan
status: open
priority: high
depends_on: [S-043]
tickets: [T-044-01, T-044-02]
---

## Goal

Suggested or user-drawn zones render as semi-transparent colored overlays on the scan terrain. The landscaper sees their design *on* the real site, not on an abstract plan view.

## Acceptance Criteria

- Zone polygons rendered as extruded meshes on top of terrain
- Semi-transparent fill with solid outline (zone type → color)
- Zone labels visible as floating text or billboards
- Zones from gap measurement (BAML suggestions) auto-placed
- User-drawn zones (from zone editor) also renderable
