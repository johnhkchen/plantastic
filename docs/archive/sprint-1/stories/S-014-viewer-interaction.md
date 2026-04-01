---
id: S-014
epic: E-006
title: Viewer Interaction
status: open
priority: high
dependencies:
  - S-013
---

# S-014: Viewer Interaction

## Purpose

Add the core interactions that make the 3D viewer useful: orbit camera, tap-to-inspect, tier toggling. These are what let a landscaper present the design and a client explore it.

## Scope

- Orbit camera: drag to rotate, scroll to zoom, pinch on touch devices
- Tap a mesh/zone → highlight it, send zone ID to SvelteKit host via postMessage
- SvelteKit receives zone ID → shows material info panel alongside viewer
- Tier toggle: host sends "show tier X" message → viewer swaps to that scene's glTF
- Sunlight direction: basic directional light that the host can adjust (slider → angle)

## Tickets

- T-014-01: Orbit camera + tap-to-inspect
- T-014-02: Tier toggle + sunlight control via host messages
