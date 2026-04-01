---
id: T-015-02
story: S-015
title: mesh-generation-export
type: task
status: open
priority: critical
phase: done
depends_on: [T-015-01]
---

## Context

Second half of the scan pipeline: take the filtered ground points from T-015-01 and produce the three output artifacts — terrain mesh (glTF), plan view image (PNG), and metadata (JSON).

## Acceptance Criteria

- Delaunay triangulation of ground points → triangle mesh
- Mesh decimation to configurable target (default ~50k triangles)
- Vertex colors preserved through triangulation if available
- glTF binary (.glb) export of terrain mesh
- Top-down orthographic projection → PNG image (plan view)
  - Configurable resolution (pixels per foot)
  - Ground colored by elevation or vertex color
  - Above-ground points optionally rendered as darker overlay (tree canopy shadows)
- Metadata JSON: bounding box (min/max x,y,z), elevation range, original point count, decimated triangle count, processing time
- End-to-end test: PLY → (terrain.glb, planview.png, metadata.json)
- The generated glTF loads in the Bevy viewer from T-013-01 (cross-validate)
- S.1.1 scenario registered and passing at ★☆☆☆☆
- Claim milestone: "pt-scan: PLY parsing + mesh generation"
