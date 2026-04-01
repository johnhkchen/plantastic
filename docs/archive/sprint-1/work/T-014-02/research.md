# T-014-02 Research — Tier Toggle + Sunlight Control

## Ticket Goal

Two features that make the 3D viewer a sales tool:
1. **Tier toggle**: host sends `setTier(name)` → viewer swaps glTF scenes (good/better/best)
2. **Sunlight slider**: host sends `setLightAngle(degrees)` → viewer rotates directional light, reports angle back

## Existing Codebase Map

### Bevy Viewer (`apps/viewer/src/`)

| File | Purpose | Relevant to this ticket |
|------|---------|------------------------|
| `main.rs` | App entry, plugin composition | Register new tier plugin |
| `bridge.rs` | postMessage ↔ Bevy messages | Add `SetTierCommand` message, add `lightAngleChanged` outbound |
| `scene.rs` | glTF loading + spawning | Refactor to support tier-based scene swapping |
| `lighting.rs` | DirectionalLight + ambient | Add outbound light angle reporting |
| `camera.rs` | Orbit camera (PanOrbitCamera) | Camera must preserve position across tier swaps |
| `picking.rs` | Tap detection + highlighting | Selection state must survive tier swaps |

### SvelteKit Host (`web/src/`)

| File | Purpose | Relevant to this ticket |
|------|---------|------------------------|
| `lib/components/viewer/types.ts` | Protocol type definitions | Add `lightAngleChanged` outbound type |
| `lib/components/viewer/Viewer.svelte` | iframe bridge component | Add `onLightAngleChanged` callback, expose `setTier` already exists |
| `routes/(app)/project/[id]/viewer/+page.svelte` | Viewer page | Add tier toggle buttons + sunlight slider UI |

### PostMessage Protocol (current state)

**Inbound (Host → Viewer):**
- `loadScene { url }` → `LoadSceneCommand` → scene.rs loads glTF
- `setLightAngle { degrees }` → `SetLightAngleCommand` → lighting.rs rotates light
- `setTier { tier }` → **parsed in JSON but ignored** — no Bevy handler

**Outbound (Viewer → Host):**
- `ready` — viewer initialized or scene loaded
- `error { message }` — failure
- `zoneTapped { zoneId }` — mesh tapped

### Bridge Architecture

`bridge.rs` uses a thread-local `VecDeque<String>` queue. JS `message` listener pushes raw JSON strings. `drain_messages` system (PreUpdate) deserializes and writes typed Bevy messages. `InboundMessage` struct has flat optional fields: `url`, `degrees`. Currently missing: `tier` field.

### Scene Loading Flow

`scene.rs` has a `SceneState` resource with `handle: Option<Handle<Gltf>>` and `spawned: bool`. On `LoadSceneCommand`:
1. Despawn all entities with `ViewerScene` marker
2. `asset_server.load(&url)` → store handle
3. `track_scene_load` polls `Assets<Gltf>` until ready, spawns `SceneRoot`
4. Sends `ready` back to host

Key observation: scene loading is already URL-based and supports despawn/reload. Tier switching = loading a different URL. The scene.rs infrastructure handles this naturally.

### Lighting State

`lighting.rs` spawns a `DirectionalLight` at -45° pitch, 30° yaw. `handle_light_angle` responds to `SetLightAngleCommand` by setting yaw while keeping -45° pitch fixed. Currently **no outbound message** reports the light angle back to the host.

### Camera Preservation

`camera.rs` spawns `PanOrbitCamera` once at startup. It is NOT part of the scene graph — it's a separate entity. Scene despawn/respawn does not affect the camera. Camera position will naturally be preserved across tier swaps because `ScenePlugin` only despawns entities with the `ViewerScene` marker.

### Selection State on Tier Swap

`picking.rs` tracks `SelectedZone { entity, original_material }`. On tier swap, the selected entity gets despawned (it's part of the old scene). The `SelectedZone` resource will hold a stale `Entity`. This needs cleanup — either clear selection on scene swap, or restore selection by zone name after new scene loads.

### Viewer Page (current)

`+page.svelte` is minimal: loads a hardcoded test scene URL, displays tapped zone name. No tier toggle UI, no sunlight slider. The `Viewer.svelte` component already exports `setTier()` and `setLightAngle()` methods that call `sendCommand()`.

## Constraints and Assumptions

1. **No pt-scene yet** — there's no scene generator that produces per-tier glTF files. For now, tier switching loads different URLs provided by the host. The host is responsible for knowing which URLs correspond to which tiers.

2. **Scene swap must not flash blank** — acceptance criteria require "fade or quick cut, not a blank frame." Bevy's asset loading is async; there will be frames between despawn and spawn where nothing renders.

3. **Light angle → time of day mapping** — the host needs the current angle to display "2:00 PM." This is a UI concern — the viewer just reports degrees, the host maps to time.

4. **WASM single-threaded** — thread-local queue pattern is safe. No concurrency concerns.

5. **Bevy 0.18 Messages API** — uses `add_message`, `MessageWriter`, `MessageReader`. These are fire-and-forget, frame-buffered.

## Key Risks

1. **Scene transition visual gap**: Between despawn and new scene spawn, the canvas shows only the light and camera. Options: crossfade (complex), keep old scene until new loads (simpler), or overlay (host-side).
2. **Selection invalidation**: Stale entity in `SelectedZone` after despawn could cause panics if queried. Must clear on scene swap.
3. **Light angle feedback frequency**: Sending a postMessage every frame while the slider moves could be noisy. May need throttling or only send on command receipt.

## Scenario Impact

S.2.4 ("3D preview per tier") is currently TwoStar. This ticket doesn't advance to ThreeStar (that requires pt-scene generating real glTF from zones + materials). But it completes the viewer-side infrastructure so that when pt-scene exists, tier toggling Just Works.

## Files to Modify (preliminary)

**Bevy (apps/viewer/src/):**
- `bridge.rs` — add `SetTierCommand`, add `tier` field to `InboundMessage`, add `send_light_angle_changed()` outbound
- `scene.rs` — refactor to support tier-aware loading (keep old scene visible until new loads)
- `lighting.rs` — send `lightAngleChanged` outbound after handling `SetLightAngleCommand`

**SvelteKit (web/src/):**
- `lib/components/viewer/types.ts` — add `lightAngleChanged` outbound message type
- `lib/components/viewer/Viewer.svelte` — add `onLightAngleChanged` callback, handle new message
- `routes/(app)/project/[id]/viewer/+page.svelte` — add tier toggle buttons + sunlight slider
