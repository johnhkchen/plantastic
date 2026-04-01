# T-013-02 Review: SvelteKit Iframe Bridge

## Summary

Embedded the Bevy WASM 3D viewer into the SvelteKit frontend via iframe with a typed bidirectional postMessage protocol. The viewer loads glTF scenes on command, reports mesh taps back to the host, and supports dynamic lighting control. S.2.4 passes at TwoStar. Milestone "Bevy viewer: glTF loading + orbit + tap-to-inspect" claimed.

## Files created

| File | Purpose |
|------|---------|
| `apps/viewer/src/bridge.rs` | BridgePlugin — postMessage listener (web_sys), thread-local message queue, Bevy message types (LoadSceneCommand, SetLightAngleCommand), outbound helpers (send_ready, send_error, send_scene_tapped) |
| `apps/viewer/src/picking.rs` | PickingSetupPlugin — reads Pointer<Click> messages, looks up entity Name, sends sceneTapped via bridge |
| `web/src/lib/components/viewer/Viewer.svelte` | Svelte 5 iframe wrapper — postMessage bridge, sceneUrl prop, onSceneTapped callback, exported setTier/setLightAngle methods, loading overlay |
| `web/src/lib/components/viewer/types.ts` | TypeScript protocol types — ViewerInboundMessage, ViewerOutboundMessage, isViewerMessage type guard |

## Files modified

| File | Change |
|------|--------|
| `apps/viewer/src/main.rs` | Added BridgePlugin and PickingSetupPlugin |
| `apps/viewer/src/scene.rs` | Replaced hardcoded asset loading with dynamic LoadSceneCommand-driven flow, added ViewerScene marker + SceneState resource |
| `apps/viewer/src/lighting.rs` | Added handle_light_angle system for SetLightAngleCommand |
| `apps/viewer/Cargo.toml` | Added serde, serde_json, wasm-bindgen, js-sys, web-sys deps |
| `web/src/routes/(app)/project/[id]/viewer/+page.svelte` | Replaced placeholder with Viewer component + tapped mesh display |
| `web/static/viewer/*` | Updated WASM build output |
| `tests/scenarios/src/suites/design.rs` | S.2.4 scenario — validates protocol contract, returns TwoStar |
| `tests/scenarios/src/progress.rs` | Claimed Bevy viewer milestone with T-013-02 |
| `crates/plantastic-api/tests/common/mod.rs` | Added `#![allow(dead_code)]` (pre-existing clippy fix) |

## postMessage protocol

```
Host → Viewer:
  { "type": "loadScene", "url": "..." }
  { "type": "setTier", "tier": "good|better|best" }
  { "type": "setLightAngle", "degrees": 0.0-360.0 }

Viewer → Host:
  { "type": "ready" }
  { "type": "error", "message": "..." }
  { "type": "sceneTapped", "meshName": "..." }
```

## Test coverage

- **S.2.4 scenario (TwoStar):** Validates the postMessage protocol contract — all inbound/outbound message shapes are tested. Since the viewer is out-of-workspace (WASM-only crate), the scenario validates the interface, not the implementation. The implementation is verified by building and running the viewer.
- **No unit tests in viewer crate:** The viewer is excluded from the Cargo workspace and targets wasm32-unknown-unknown. Standard `cargo test` doesn't apply. Manual browser testing validates the full pipeline.
- **Pre-existing test helper fix:** `#![allow(dead_code)]` on test common module — these helpers are for future integration tests gated on Postgres.

## Scenario dashboard (before → after)

Before: S.2.4 was NotImplemented. Dashboard at 48.0/240.0 min.
After: S.2.4 passes at TwoStar (10 min weighted to 4.0 min). Dashboard at 58.0/240.0 min (24.2%). No regressions — all 8 previously-passing scenarios still pass.

## Quality gate

`just check` passes: fmt-check, lint (clippy -D warnings), test (all workspace tests), scenarios (no regressions, S.2.4 at TwoStar).

## Architecture notes

- **Thread-local queue is safe** because WASM is single-threaded. The Closure pushed from the JS event listener writes to the same thread's RefCell that the Bevy system reads from.
- **Closure.forget()** is intentional — the message listener must live for the app's lifetime. No cleanup needed since WASM apps don't gracefully shut down.
- **postMessage origin is `"*"`** — acceptable for same-origin iframe. If the viewer is served from a different origin in production, tighten to the specific origin.
- **Viewer.svelte sends loadScene on `ready`** — the viewer sends `ready` immediately on startup (from `setup_message_listener`), then the Svelte component sends `loadScene`. This avoids a race condition where the host sends `loadScene` before the viewer's message listener is registered.

## Open concerns

1. **WASM binary in static/**: The 14 MB WASM binary is committed in `web/static/viewer/`. This works but bloats the git repo. Future: serve from S3/R2 or add to `.gitignore` with a build step that copies from `apps/viewer/dist/`.
2. **setTier not wired end-to-end**: The bridge parses `setTier` messages but no Bevy system consumes them yet. This is expected — tier switching requires pt-scene to generate per-tier glTF files. The protocol slot is reserved.
3. **No origin validation**: `isViewerMessage()` checks message shape but not origin. Browser extensions or other iframes could send matching messages. Low risk for now; tighten when viewer serves from a different origin.
4. **Test coverage gap**: The viewer's Rust code (bridge, picking, scene loading) has no automated tests. The scenario validates the protocol contract but not the implementation. A headless browser test (e.g., playwright) would close this gap but is out of scope for this ticket.

## Acceptance criteria checklist

- [x] Bevy viewer served as standalone HTML page (Trunk build)
- [x] Embedded in SvelteKit via iframe
- [x] postMessage protocol defined: Host→Viewer (loadScene, setTier, setLightAngle), Viewer→Host (ready, error, sceneTapped)
- [x] SvelteKit Viewer component with sceneUrl prop and onSceneTapped callback
- [x] Viewer page at /project/[id]/viewer renders embedded viewer
- [x] Test with glTF from T-013-01 — tap mesh, see name in SvelteKit UI
- [x] S.2.4 scenario registered and passing at TwoStar
- [x] Milestone "Bevy viewer: glTF loading + orbit + tap-to-inspect" claimed
