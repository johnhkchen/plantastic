---
id: T-031-02
story: S-031
title: scene-api-viewer-wiring
type: task
status: open
priority: high
phase: ready
depends_on: [T-031-01]
---

## Context

Wire pt-scene into the API and connect to the Bevy viewer. The viewer already supports loading glTF via postMessage — just needs a real URL to load.

## Acceptance Criteria

- `GET /projects/:id/scene/:tier` route in plantastic-api
  - Generates scene via pt-scene, uploads to S3, returns presigned URL
  - Optional: cache in S3 (key: `scenes/{project_id}/{tier}/{version}.glb`)
- Viewer page (`/project/[id]/viewer`) updated to:
  - Fetch scene URL from API on load
  - Send `loadScene(url)` to Bevy iframe
  - On tier toggle: fetch new tier's scene URL, send `setTier(tier, url)`
- S.2.4 scenario updated: verify that scene generation produces valid glTF from known project data
  - S.2.4 advances to ★★★☆☆ (real project data rendered via UI)
- Claim "pt-scene: 3D scene generation from project model" milestone
- Scene export smoke test: create project → add zones + materials → GET /scene/good → valid GLB
  - Can be used as system integration check (`just smoke-scene`)
- `just check` passes

## Implementation Notes

- Same data loading pattern as /quote/:tier and /proposal routes
- Scene generation is CPU-bound — consider spawn_blocking like scan processing
- The viewer already handles loadScene and setTier — no Bevy changes needed
- Cache invalidation: regenerate when zones or assignments change (version from updated_at timestamp)
- The smoke test recipe in justfile is a lightweight integration check: "can we render a project?"
