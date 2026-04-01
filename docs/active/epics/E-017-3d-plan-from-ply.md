---
id: E-017
title: 3D Plan View from PLY Scan
status: open
priority: critical
sprint: 3
---

## Context

The scan pipeline produces classified features, measured gaps, and suggested planter zones. The Bevy viewer can load glTF. But the two aren't connected for the scan-to-design flow: a landscaper scans a site and should see their plan overlaid on the real terrain in 3D.

This epic bridges: classified scan → zone suggestions → 3D scene with terrain + zones + materials rendered together. The "wow" demo: scan Powell & Market → see two trunks on brick path → suggested planter zone appears between them → toggle Good/Better/Best → orbit the scene.

## Architecture

```
Classified scan (terrain GLB + feature candidates + gaps)
  + Zone suggestions (from BAML or user-drawn)
  + Material assignments (per tier)
        │
        ▼
  pt-scene: compose_scan_scene()
    - Terrain mesh as base layer (vertex-colored from scan RGB)
    - Feature bounding boxes as wireframe overlays
    - Zone polygons extruded with material colors
    - Per-tier switching via setTier postMessage
        │
        ▼
  Bevy viewer renders composite scene
```

## Stories

- S-043: Scan Terrain in Viewer (terrain GLB from scan loads in Bevy)
- S-044: Zone Overlay on Scan (suggested/drawn zones rendered on terrain)
- S-045: Tier Switching on Scan Scene (Good/Better/Best material preview)

## Success Criteria

- Powell & Market terrain renders in Bevy viewer with vertex colors (brick texture visible)
- Suggested planter zone between trunks visible as colored overlay
- Tier toggle swaps planter materials (different colors/textures per tier)
- S.2.4 advances to ★★★★☆ (real scan data + zone overlay + tier switching)
- S.4.1 advances to ★☆☆☆☆+ (viewer works on tablet Safari with scan data)
