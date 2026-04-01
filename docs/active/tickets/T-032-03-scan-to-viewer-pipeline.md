---
id: T-032-03
story: S-032
title: scan-to-viewer-pipeline
type: task
status: open
priority: high
phase: done
depends_on: [T-032-01]
---

## Context

The end-to-end proof: PLY file → pt-scan → pt-scene → glTF → Bevy viewer. This wires the existing scan output (terrain mesh from ground points) into pt-scene so it renders in the viewer. Classified features (from T-033/T-034) will be added as named meshes later, but the terrain ground plane should render immediately.

## Acceptance Criteria

- CLI example or `just` recipe: `just scan-to-viewer <ply-path>`
  1. process_scan() → ClassifiedCloud
  2. generate_terrain() → GLB (ground mesh)
  3. Write GLB to a local path the viewer can load
  4. Print instructions to open the viewer page with the GLB URL
- pt-scene's `generate_scene()` can accept a terrain GLB as the base layer
- The Bevy viewer loads and renders the terrain mesh with orbit camera
- Powell & Market: brick paths visible as ground plane, trunks visible as obstacle geometry
- Later (T-033+): classified features become named mesh nodes (tappable in viewer)

## Implementation Notes

- The viewer already loads arbitrary glTF via `loadScene(url)` postMessage
- For local dev: serve the GLB from a file:// URL or a tiny HTTP server
- The terrain GLB from generate_terrain() already has POSITION, NORMAL, COLOR_0 — it should render with vertex colors in Bevy
- This proves: "I scanned a real place with my iPhone and now I can orbit it in 3D in the browser"
- The scan-to-viewer path is the demo that sells the product
