---
id: S-043
epic: E-017
title: Scan Terrain in Viewer
status: open
priority: critical
tickets: [T-043-01, T-043-02]
---

## Goal

The terrain GLB from scan processing loads in the Bevy viewer with vertex colors intact. This is the "I scanned this and now I'm orbiting it in 3D" moment.

## Acceptance Criteria

- Scan terrain GLB (from process_sample) renders in Bevy viewer
- Vertex colors from scan RGB visible (brick texture, not flat gray)
- Orbit, zoom, pan work on the terrain
- Feature bounding boxes optionally rendered as wireframe overlays
- Works from both local file and S3 presigned URL
