---
id: T-013-02
story: S-013
title: sveltekit-iframe-bridge
type: task
status: open
priority: critical
phase: done
depends_on: [T-013-01]
---

## Context

Embed the Bevy WASM viewer in the SvelteKit app and establish bidirectional communication. The viewer runs in an iframe; the host page controls it (load scene, switch tier, set lighting) and receives events (zone tapped, camera moved).

## Acceptance Criteria

- Bevy viewer served as a standalone HTML page (built by trunk or wasm-pack)
- Embedded in SvelteKit project workspace page via iframe
- postMessage protocol defined and documented:
  - Host → Viewer: loadScene(url), setTier(name), setLightAngle(degrees)
  - Viewer → Host: zoneTapped(zoneId), ready, error(message)
- SvelteKit Viewer component: `<Viewer sceneUrl={url} on:zoneTapped={handler} />`
- Viewer page at /project/[id]/viewer renders the embedded viewer
- Test with the glTF from T-013-01 — tap a mesh, see its name in the SvelteKit UI
- S.2.4 scenario registered and passing at ★★ (viewer loads scene + responds to tap)
- Claim milestone: "Bevy viewer: glTF loading + orbit + tap-to-inspect"
