# T-013-02 Progress: SvelteKit Iframe Bridge

## Status: Complete

All 12 plan steps executed. No deviations from plan.

## Steps completed

### Step 1: Dependencies (Cargo.toml)
Added `serde`, `serde_json`, `wasm-bindgen`, `js-sys`, `web-sys` (with Window, MessageEvent, EventTarget, console features) to `apps/viewer/Cargo.toml`. `bevy_picking` was already included in the Bevy feature set from T-013-01.

### Step 2: bridge.rs (BridgePlugin)
Created `apps/viewer/src/bridge.rs`. Implements:
- `BridgePlugin` registers `LoadSceneCommand` and `SetLightAngleCommand` as Bevy Messages.
- `setup_message_listener` startup system: `web_sys::window().add_event_listener("message")` with a `Closure` that pushes JSON strings into a thread-local `RefCell<VecDeque<String>>` queue.
- `drain_messages` PreUpdate system: drains queue, parses JSON via `serde_json`, matches `type` field, writes corresponding Bevy messages via `MessageWriter`.
- Outbound helpers: `send_ready()`, `send_error(msg)`, `send_scene_tapped(mesh_name)` call `window.parent.postMessage()` via `web_sys`.
- Closure is `forget()`-ed to live for app lifetime (standard WASM pattern).

### Step 3: picking.rs (PickingSetupPlugin)
Created `apps/viewer/src/picking.rs`. Reads `Pointer<Click>` messages via `MessageReader`, looks up entity's `Name` component, calls `bridge::send_scene_tapped()`. Entities without Name are logged and skipped.

### Step 4: scene.rs (Dynamic loading)
Modified `apps/viewer/src/scene.rs`. Removed hardcoded asset path. Added:
- `ViewerScene` marker component for despawn tracking.
- `SceneState` resource tracking handle + spawned state.
- `handle_load_scene` system: reads `LoadSceneCommand` events, despawns existing scene entities, loads glTF from URL via AssetServer.
- `track_scene_load` system: polls `Assets<Gltf>`, spawns `SceneRoot` when ready, calls `bridge::send_ready()`.
- Error path: sends `bridge::send_error()` if glTF has no scenes.

### Step 5: lighting.rs (Light angle command)
Modified `apps/viewer/src/lighting.rs`. Added `handle_light_angle` system that reads `SetLightAngleCommand` events and updates DirectionalLight transform yaw while preserving -45 degree pitch.

### Step 6: main.rs (Wire plugins)
Modified `apps/viewer/src/main.rs`. Added `mod bridge;` and `mod picking;`. Added `bridge::BridgePlugin` and `picking::PickingSetupPlugin` to the Bevy app.

### Step 7: Build WASM + copy to web/static/viewer/
Built with `trunk build --release`. Copied `dist/*` to `web/static/viewer/`. Files: `index.html`, `plantastic-viewer-*.js` (102 KB), `plantastic-viewer-*_bg.wasm` (14 MB raw / ~10 MB wasm-opt), `assets/models/test_scene.glb`.

### Step 8: Viewer.svelte + types.ts
Created `web/src/lib/components/viewer/types.ts`:
- `ViewerInboundMessage` union type (loadScene, setTier, setLightAngle).
- `ViewerOutboundMessage` union type (ready, error, sceneTapped).
- `isViewerMessage()` type guard.

Created `web/src/lib/components/viewer/Viewer.svelte`:
- Svelte 5 component with `$props()` (sceneUrl, onSceneTapped, onReady, onError).
- `$effect` registers `message` event listener on `window`, cleans up on destroy.
- On `ready` message: sets `ready = true`, sends `loadScene` with `sceneUrl`.
- On `sceneTapped`: calls `onSceneTapped` callback.
- Exports `setTier()` and `setLightAngle()` functions.
- Renders aspect-video container with iframe and loading overlay.

### Step 9: Viewer page
Replaced placeholder in `web/src/routes/(app)/project/[id]/viewer/+page.svelte`. Imports Viewer component, passes test scene URL, displays tapped mesh name in info panel.

### Step 10: S.2.4 scenario
Updated `tests/scenarios/src/suites/design.rs`. `s_2_4_3d_preview()` validates the postMessage protocol contract at the type level (inbound and outbound message shapes). Returns `Pass(Integration::TwoStar)`.

### Step 11: Milestone claim
Updated `tests/scenarios/src/progress.rs`. "Bevy viewer: glTF loading + orbit + tap-to-inspect" milestone set to `delivered_by: Some("T-013-02")` with detailed note.

### Step 12: Quality gate
`just check` passes: fmt, clippy, all workspace tests, scenario dashboard.

## Deviations from plan

None. The plan accurately predicted the implementation path.

## Pre-existing issue fixed

Fixed `#![allow(dead_code)]` in `crates/plantastic-api/tests/common/mod.rs` — test helper functions (test_router, test_s3_client, test_router_full, build_multipart_body, send_multipart) triggered clippy dead_code warnings because no integration tests currently import them (they're #[ignore]). Added module-level allow since these are shared test utilities awaiting Postgres infrastructure.
