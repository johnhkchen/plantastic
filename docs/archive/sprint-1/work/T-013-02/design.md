# T-013-02 Design: SvelteKit Iframe Bridge

## Decision: postMessage with Rust-side web_sys listener

### Option A: JS-only bridge (postMessage in index.html `<script>`)

Put the message listener in a `<script>` block in `index.html`. JS receives postMessage, calls `window.wasmBindings.load_scene(url)` etc. Bevy exports `#[wasm_bindgen]` functions.

**Pros:** No Rust-side web_sys dependency. Clean separation — JS handles messaging, Rust handles rendering.
**Cons:** Two-step indirection (postMessage → JS → wasm_bindgen → Bevy Event). Trunk overwrites index.html on build — custom scripts in the template are preserved, but the generated output moves them. Harder to test. Need to coordinate `window.wasmBindings` readiness.

### Option B: Rust-side web_sys postMessage listener

Add `web_sys` + `js_sys` + `wasm-bindgen` deps (already transitive deps of Bevy WASM). Create a Bevy plugin (`BridgePlugin`) that:
1. On startup: registers a `message` event listener on `window` via `web_sys`.
2. Listener pushes commands into a thread-local queue (or a `web_sys::MessagePort`).
3. Bevy system drains the queue each frame, converts JSON to Bevy Events.
4. Other systems react to events (load scene, change lighting, etc.).
5. Outbound: Rust calls `window.parent.postMessage()` via web_sys for ready/error/zoneTapped.

**Pros:** All logic in Rust. Single codebase. Bevy's event system handles everything. No JS coordination issues.
**Cons:** `web_sys` adds complexity. Thread-local static for the queue (WASM is single-threaded, so this is safe). Slightly more Rust code.

### Option C: Custom DOM events (no iframe, same page)

Skip the iframe. Load WASM directly into the SvelteKit page. Use CustomEvents for communication.

**Pros:** No iframe overhead. Simpler DOM.
**Cons:** Bevy takes over the full window — conflicts with SvelteKit's DOM. Canvas sizing fights with SvelteKit layout. Can't isolate Bevy's event handling (it captures all keyboard/mouse). The iframe sandbox is a feature, not a workaround.

### Decision: Option B — Rust-side web_sys

**Why:** Single language for all viewer logic. The web_sys/js_sys crates are already in the dependency tree (Bevy WASM pulls them in). Thread-local queue is idiomatic for WASM. The BridgePlugin pattern keeps all interop in one module with clean Bevy Event types. Future commands (setTier, measurements, etc.) just add new event variants.

## postMessage Protocol

All messages are JSON with a `type` field. Payload fields vary by type.

### Host → Viewer (inbound)

```json
{ "type": "loadScene", "url": "https://cdn.example.com/scenes/abc.glb" }
{ "type": "setTier", "tier": "better" }
{ "type": "setLightAngle", "degrees": 45.0 }
```

### Viewer → Host (outbound)

```json
{ "type": "ready" }
{ "type": "error", "message": "Failed to load scene" }
{ "type": "sceneTapped", "meshName": "patio_travertine" }
```

Note: renamed from `zoneTapped` to `sceneTapped` since the viewer deals in meshes, not domain zones. The SvelteKit component maps mesh names to zone IDs.

## Tap detection approach

Use Bevy 0.18's built-in `MeshPickingPlugin` (from `bevy_picking`). This is part of Bevy's default plugins and provides `Pointer<Click>` observers on entities with meshes. When a mesh is clicked/tapped, the observer fires with the entity. We look up the entity's `Name` component (set by glTF import) and send it via postMessage.

If `MeshPickingPlugin` isn't available with our feature set, fallback: manual raycast using screen-to-ray conversion + mesh intersection. But picking is included in `bevy_picking` which is part of DefaultPlugins.

**Feature requirement:** Add `bevy_picking` to the Bevy features list.

## Scene loading from URL

The current `scene.rs` hardcodes `asset_server.load("models/test_scene.glb")`. For dynamic loading:

1. BridgePlugin receives `loadScene` message with URL.
2. Sends `LoadSceneEvent { url }` Bevy event.
3. ScenePlugin system handles the event:
   - Despawn current scene entities.
   - Load new scene from URL via AssetServer.
   - Track loading state, send `ready` when spawned.

For WASM, Bevy's AssetServer can load from absolute HTTP URLs. The AssetServer uses `fetch()` under the hood in WASM builds.

## SvelteKit Viewer component

```
web/src/lib/components/viewer/Viewer.svelte
```

Props:
- `sceneUrl: string` — URL to the glTF scene file.
- `onSceneTapped: (meshName: string) => void` — callback when a mesh is tapped.

Behavior:
- Renders an iframe pointing to the viewer's index.html.
- On mount: listens for `message` events from the iframe.
- When iframe sends `ready`: sends `loadScene` with `sceneUrl`.
- When iframe sends `sceneTapped`: calls `onSceneTapped`.
- Provides methods via `$bindable` or exposed functions: `setTier()`, `setLightAngle()`.

## Viewer page

```
web/src/routes/(app)/project/[id]/viewer/+page.svelte
```

- Imports `Viewer` component.
- Passes `sceneUrl` (for now, a test URL pointing to `test_scene.glb`).
- Displays tapped mesh name in a panel below/beside the viewer.
- Aspect ratio container: 16:9 with `aspect-video` Tailwind class.

## Serving the viewer

For this spike: the built viewer dist files are committed and served as static assets. The SvelteKit app's iframe src points to `/viewer/index.html`. We add the viewer's dist output to `web/static/viewer/` (SvelteKit serves `static/` at the root).

Alternative: serve from a separate origin. But same-origin is simpler for the spike and avoids CORS for asset loading.

**Build flow:** `cd apps/viewer && trunk build --release` → copy `dist/*` to `web/static/viewer/`.

## S.2.4 scenario upgrade

The S.2.4 test currently returns `NotImplemented`. After this ticket:
- The Bevy viewer loads a glTF scene via postMessage.
- Tap detection reports mesh names back to the host.
- The SvelteKit page renders the viewer and displays tap results.

This is ★★ integration: the viewer works in isolation (★) and is embedded in the SvelteKit UI with bidirectional communication (★★). No API persistence or real scene generation yet (that's pt-scene).

## Milestone claim

"Bevy viewer: glTF loading + orbit + tap-to-inspect" — delivered by T-013-02.
- What: BridgePlugin (postMessage), MeshPickingPlugin (tap), Viewer.svelte (iframe), viewer page
- Unlocks: S.4.1, S.4.3 (both still need pt-scene for real scenes)
- What's still needed: pt-scene for actual zone→glTF generation
