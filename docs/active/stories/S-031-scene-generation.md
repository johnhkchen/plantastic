---
id: S-031
epic: E-013
title: Scene Generation
status: open
priority: high
tickets: [T-031-01, T-031-02]
---

## Goal

Create pt-scene crate that generates glTF scenes from project zones + material assignments. Wire into the viewer and use scene export as an early system integration check.

## Acceptance Criteria

- pt-scene crate: generate_scene(zones, assignments, materials, tier) → SceneOutput (glb bytes + metadata)
- Zone polygons → extruded 3D meshes (height from material extrusion behavior)
- Material PBR properties mapped to glTF materials (color from category, texture_ref when available)
- API route: GET /projects/:id/scene/:tier → presigned S3 URL to .glb
- S.2.4 advances to ★★★☆☆ (real project data rendered in viewer)
- Scene export usable as a system integration smoke test
