---
id: T-014-01
story: S-014
title: orbit-camera-tap-inspect
type: task
status: open
priority: high
phase: done
depends_on: [T-013-02]
---

## Context

Core viewer interactions: orbit camera and tap-to-inspect. These make the 3D view useful — a landscaper can rotate around the design and tap zones to see details.

## Acceptance Criteria

- Orbit camera: drag to rotate around a focus point, scroll to zoom, right-drag to pan
- Touch support: single finger to rotate, pinch to zoom, two-finger drag to pan
- Tap/click a mesh → raycast to identify which zone → send zoneTapped(zoneId) to host
- Tapped zone highlights (outline or color change)
- Camera bounds: prevent camera from going underground or too far away
- Smooth damping on camera movement (not snappy)
- 60 FPS on desktop, 30+ FPS on iPad
