---
id: T-013-01
story: S-013
title: bevy-wasm-gltf-loading
type: spike
status: open
priority: critical
phase: done
depends_on: []
---

## Context

This is a spike to prove Bevy compiles to WASM and can load/render a glTF scene in the browser. This is the highest-risk technical unknown in the frontend stack. If Bevy→WASM doesn't work at acceptable performance (binary size, load time, frame rate, iPad Safari), we need to know now — not after building a scene generator.

## Acceptance Criteria

- Bevy app in apps/viewer/ compiles to WASM (via wasm-pack or trunk)
- Loads a test glTF file (grab any free model — a simple house, terrain, or furniture)
- Renders with PBR materials and directional lighting
- Measures and documents: .wasm binary size, time to first frame, steady-state FPS
- Tests in Chrome, Firefox, Safari desktop
- Tests on iPad Safari (real device or BrowserStack) — document any issues
- If binary size > 20 MB, investigate wasm-opt, feature gating, or alternative approaches
- If iPad Safari fails, document the failure mode and propose alternatives
- Write up findings in research.md for the ticket work artifacts
