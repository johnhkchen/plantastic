# T-013-02 Plan: SvelteKit Iframe Bridge

## Step 1: Add dependencies to viewer Cargo.toml

Add `serde`, `serde_json`, `wasm-bindgen`, `web-sys`, `js-sys` to `apps/viewer/Cargo.toml`. Add `bevy_picking` to Bevy features if needed. Verify `cargo check` passes (not `cargo build` — just checking the dep graph).

**Verify:** `cd apps/viewer && cargo check --target wasm32-unknown-unknown` succeeds.

## Step 2: Implement bridge.rs — BridgePlugin

Create `apps/viewer/src/bridge.rs`:
- Define Bevy Event types: `LoadSceneCommand { url: String }`, `SetLightAngleCommand { degrees: f32 }`.
- Thread-local `RefCell<VecDeque<String>>` message queue.
- `setup_message_listener` startup system: uses `web_sys::window()` to add `message` event listener. Closure pushes `event.data()` JSON string into the queue.
- `drain_messages` update system: drains queue, parses JSON with `serde_json`, matches on `type` field, sends corresponding Bevy event.
- `send_to_host(type, payload)`: calls `window.parent.postMessage()`.
- `send_ready()`, `send_error(msg)`, `send_scene_tapped(name)` convenience functions.

**Verify:** Compiles with `cargo check --target wasm32-unknown-unknown`.

## Step 3: Implement picking.rs — Tap-to-inspect

Create `apps/viewer/src/picking.rs`:
- `PickingSetupPlugin` that adds an observer for `Pointer<Click>`.
- Observer callback: get clicked entity, query for `Name` component, call `bridge::send_scene_tapped(&name)`.
- Handle entities without Name gracefully (log warning, skip).

**Verify:** Compiles.

## Step 4: Update scene.rs — Dynamic scene loading

Modify `apps/viewer/src/scene.rs`:
- Remove the hardcoded `load_scene` startup system.
- Add `handle_load_scene` system that reads `EventReader<LoadSceneCommand>`.
- On event: despawn all entities with `SceneRoot` component, load new glTF from the URL.
- Modify `track_scene_load` to send `bridge::send_ready()` when scene is spawned.
- Add `SceneState` resource tracking: `Empty`, `Loading`, `Loaded`.

**Verify:** Compiles.

## Step 5: Update lighting.rs — Light angle command

Modify `apps/viewer/src/lighting.rs`:
- Add `handle_light_angle` system that reads `EventReader<SetLightAngleCommand>`.
- On event: query for `DirectionalLight`, update transform yaw to `degrees`.

**Verify:** Compiles.

## Step 6: Update main.rs — Wire plugins

Modify `apps/viewer/src/main.rs`:
- Add `mod bridge;` and `mod picking;`.
- Add `bridge::BridgePlugin` and `picking::PickingSetupPlugin` to app.
- Keep existing plugins.

**Verify:** `cargo check --target wasm32-unknown-unknown` succeeds for full app.

## Step 7: Build WASM and copy to web/static/viewer/

Run `cd apps/viewer && trunk build --release`. Copy `dist/*` to `web/static/viewer/`. This makes the viewer accessible at `/viewer/index.html` from the SvelteKit dev server and in production.

**Verify:** `ls web/static/viewer/index.html` exists.

## Step 8: Create Viewer.svelte + types.ts

Create `web/src/lib/components/viewer/types.ts`:
- TypeScript types for inbound/outbound messages.
- Type guard function.

Create `web/src/lib/components/viewer/Viewer.svelte`:
- iframe wrapper with postMessage bridge.
- Props: sceneUrl, onSceneTapped, onReady, onError.
- Aspect-video container with rounded corners and dark background.

**Verify:** `cd web && npx svelte-check` passes (or at minimum no type errors in new files).

## Step 9: Update viewer page

Replace `web/src/routes/(app)/project/[id]/viewer/+page.svelte`:
- Import and render Viewer component.
- Pass test scene URL (`/viewer/assets/models/test_scene.glb`).
- Display tapped mesh name in an info panel below the viewer.

**Verify:** `cd web && npm run build` succeeds.

## Step 10: Update S.2.4 scenario

Modify `tests/scenarios/src/suites/design.rs`:
- Replace `s_2_4_3d_preview` stub with a test that validates the bridge protocol contract.
- Since the Bevy viewer is out-of-workspace, the scenario can't import viewer code directly. Instead, validate at the interface level: the message types, the SvelteKit component contract, and the expected behavior.
- Return `Pass(Integration::TwoStar)`.

**Verify:** `cargo run -p pt-scenarios` shows S.2.4 passing at ★★.

## Step 11: Claim milestone in progress.rs

Update `tests/scenarios/src/progress.rs`:
- Set "Bevy viewer: glTF loading + orbit + tap-to-inspect" delivered_by to `Some("T-013-02")`.
- Write detailed note.

**Verify:** `cargo run -p pt-scenarios` shows milestone as delivered.

## Step 12: Run quality gate

Run `just check` (fmt + lint + test + scenarios). Fix any issues.

**Verify:** All four checks pass. Scenario dashboard shows no regressions, S.2.4 at ★★.

## Testing strategy

- **Unit tests:** None in the viewer crate (it's excluded from workspace, can't run `cargo test` easily for WASM). The scenario harness validates the contract.
- **Scenario test (S.2.4):** Validates the protocol and integration level. Returns TwoStar.
- **Manual verification:** Build viewer, open `web/static/viewer/index.html` in browser, confirm scene loads, tap mesh, see postMessage in console. Then run SvelteKit dev server, navigate to `/project/{id}/viewer`, confirm iframe loads, tap mesh, see name in UI.
- **No mocks:** No crate boundaries crossed (viewer is standalone). Scenario test validates real types.
