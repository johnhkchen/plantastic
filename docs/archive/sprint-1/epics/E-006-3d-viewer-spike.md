---
id: E-006
title: 3D Viewer Foundation
status: open
sprint: 2
---

# E-006: 3D Viewer Foundation

## Goal

Prove that Bevy compiles to WASM, loads a glTF scene in a browser, and can be embedded in the SvelteKit app. Get to ★★ for S.2.4 (3D preview) and lay the groundwork for S.4.1 (crew tablet viewer).

This is a technical risk reduction epic. Bevy→WASM has known rough edges: binary size (multi-MB .wasm), load time, mobile browser compatibility (iPad Safari is a hard requirement for crew use). We need to hit these problems early, not after we've built a scene generator that produces glTF nobody can view.

## Target

- S.2.4 3D preview per tier: — → ★★ (0.0 → 4.0 effective min)
- Bevy WASM binary loads and renders in Chrome + Safari
- Embedded in SvelteKit app (iframe or web component)
- Orbit camera, basic interaction (tap to select mesh)

## Stories

- **S-013**: Bevy WASM spike — compile, load glTF, embed in SvelteKit
- **S-014**: Viewer interaction — orbit camera, tap-to-inspect, tier toggle

## Success Criteria

- Bevy app compiles to WASM under 15 MB (ideally under 10 MB)
- Loads a test glTF scene (can be any model — doesn't need to be a garden) in < 5 seconds on desktop
- Renders in Chrome, Firefox, and Safari (including iPad Safari)
- Embedded in SvelteKit page via iframe with bidirectional message passing
- Orbit camera: drag to rotate, scroll to zoom, pinch on mobile
- Tap a mesh → get its name/metadata back in the parent SvelteKit page
- S.2.4 scenario registered and passing at ★★
