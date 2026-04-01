# T-013-02 Structure: SvelteKit Iframe Bridge

## Files created

### 1. `apps/viewer/src/bridge.rs` — BridgePlugin (postMessage interop)

New Bevy plugin module. Responsibilities:
- Register `window` `message` event listener on startup via `web_sys`.
- Thread-local `RefCell<Vec<String>>` queue for inbound messages (WASM is single-threaded).
- Bevy events: `LoadSceneCommand`, `SetTierCommand`, `SetLightAngleCommand`.
- Drain system: each frame, drain queue, parse JSON, send Bevy events.
- Outbound helper: `send_to_host(msg: &str)` calls `window.parent.postMessage()`.
- Public functions: `send_ready()`, `send_error(msg)`, `send_scene_tapped(mesh_name)`.

Dependencies: `web_sys` (Window, MessageEvent, EventTarget, HtmlIFrameElement), `js_sys` (JsValue, Function, Closure), `wasm-bindgen`, `serde_json`.

### 2. `apps/viewer/src/picking.rs` — Tap-to-inspect system

New module. Responsibilities:
- Observer on `Pointer<Click>` events from Bevy's picking system.
- On click: look up entity's `Name` component.
- Call `bridge::send_scene_tapped(name)`.
- Graceful fallback if entity has no Name.

### 3. `web/src/lib/components/viewer/Viewer.svelte` — Iframe wrapper component

New Svelte 5 component. Props:
- `sceneUrl: string`
- `onSceneTapped?: (meshName: string) => void`
- `onReady?: () => void`
- `onError?: (message: string) => void`

Internal state:
- `iframeRef: HTMLIFrameElement` — bound via `bind:this`.
- `ready: boolean` — tracks viewer readiness.

Lifecycle:
- `$effect`: attach `message` event listener on `window`.
- On `ready` message: set `ready = true`, send `loadScene` command with `sceneUrl`, call `onReady`.
- On `sceneTapped` message: call `onSceneTapped(meshName)`.
- On `error` message: call `onError(message)`.
- Cleanup: remove event listener on destroy.

Methods (exported via `export function`):
- `sendCommand(type: string, payload?: Record<string, unknown>)` — generic postMessage sender.
- `setTier(tier: string)` — sends setTier command.
- `setLightAngle(degrees: number)` — sends setLightAngle command.

Rendering:
- `<div class="relative aspect-video w-full overflow-hidden rounded-lg bg-gray-900">` — container.
- `<iframe src="/viewer/index.html" class="h-full w-full border-0" title="3D Viewer" allow="autoplay" bind:this={iframeRef}>` — the viewer.

### 4. `web/src/lib/components/viewer/types.ts` — Message type definitions

TypeScript interfaces for the postMessage protocol:
- `ViewerInboundMessage` — union type for Host→Viewer messages.
- `ViewerOutboundMessage` — union type for Viewer→Host messages.
- Type guards: `isViewerMessage(event: MessageEvent)`.

## Files modified

### 5. `apps/viewer/src/main.rs` — Add BridgePlugin and picking

- Add `mod bridge;` and `mod picking;`.
- Add `.add_plugins(bridge::BridgePlugin)`.
- Add `.add_plugins(picking::PickingPlugin)`.
- Remove hardcoded scene loading (moved to bridge-driven flow).

### 6. `apps/viewer/src/scene.rs` — Dynamic scene loading

- Remove hardcoded `load_scene` startup system.
- Add system that listens for `LoadSceneCommand` event.
- On event: despawn existing scene, load new glTF from URL.
- Track loading → send `ready` via bridge when scene spawns.
- Keep `track_scene_load` but make it aware of dynamic reloads.

### 7. `apps/viewer/src/lighting.rs` — Light angle command

- Add system that listens for `SetLightAngleCommand` event.
- On event: update DirectionalLight transform with new yaw angle.

### 8. `apps/viewer/Cargo.toml` — Add dependencies

- Add `web-sys` with features: `Window`, `MessageEvent`, `EventTarget`, `console`.
- Add `wasm-bindgen` (explicit, even though transitive).
- Add `serde` + `serde_json` for JSON parsing.
- Add `js-sys` for closures.
- Add `bevy_picking` to Bevy features (if not already included in DefaultPlugins).

### 9. `apps/viewer/index.html` — No changes needed

The Rust-side bridge handles everything. No additional JS required.

### 10. `web/src/routes/(app)/project/[id]/viewer/+page.svelte` — Real viewer page

Replace placeholder with:
- Import `Viewer` component.
- Render `<Viewer>` with test scene URL.
- Show tapped mesh name in a panel.
- Project ID from layout data (for future scene loading).

### 11. `tests/scenarios/src/suites/design.rs` — S.2.4 scenario

Update `s_2_4_3d_preview()` from `NotImplemented` to a test that:
- Verifies the postMessage protocol types compile and serialize correctly.
- Verifies the Viewer component contract (sceneUrl → loadScene flow).
- Returns `Pass(Integration::TwoStar)`.

Since we can't run a real browser in the scenario harness, the test verifies the protocol contract at the type level — confirming that the bridge code exists and the message format is correct.

### 12. `tests/scenarios/src/progress.rs` — Claim milestone

Update "Bevy viewer: glTF loading + orbit + tap-to-inspect" milestone:
- `delivered_by: Some("T-013-02")`
- Note describing: BridgePlugin, PickingPlugin, Viewer.svelte, postMessage protocol.

## Files NOT changed

- `apps/viewer/src/camera.rs` — No changes. PanOrbitCamera works as-is.
- `web/src/lib/components/TabNav.svelte` — Already has Viewer tab.
- `web/src/routes/(app)/project/[id]/+layout.svelte` — No changes needed.
- `tests/scenarios/src/suites/crew_handoff.rs` — S.4.1 stays NotImplemented (needs pt-scene).

## Module dependency flow

```
Host (SvelteKit)                    Viewer (Bevy WASM, in iframe)
┌──────────────────┐                ┌─────────────────────────┐
│ Viewer.svelte    │  postMessage   │ bridge.rs (BridgePlugin)│
│  iframe ref ─────┼───────────────>│  message listener       │
│  message listener│<───────────────┤  send_to_host()         │
│                  │                │                         │
│ types.ts         │                │ scene.rs (ScenePlugin)  │
│  protocol types  │                │  LoadSceneCommand       │
│                  │                │                         │
│ +page.svelte     │                │ picking.rs              │
│  renders Viewer  │                │  Pointer<Click> → Name  │
│  shows tap info  │                │                         │
└──────────────────┘                │ lighting.rs             │
                                    │  SetLightAngleCommand   │
                                    └─────────────────────────┘
```

## Ordering

1. Cargo.toml deps (enables compilation of new modules)
2. bridge.rs (foundation — all other modules depend on bridge events)
3. picking.rs (depends on bridge for outbound messages)
4. scene.rs modifications (depends on bridge for LoadSceneCommand)
5. lighting.rs modifications (depends on bridge for SetLightAngleCommand)
6. main.rs modifications (wires plugins together)
7. Viewer.svelte + types.ts (SvelteKit side, independent of Rust build)
8. +page.svelte (depends on Viewer.svelte)
9. Scenario + milestone updates (last — requires working code)
