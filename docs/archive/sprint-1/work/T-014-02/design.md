# T-014-02 Design — Tier Toggle + Sunlight Control

## Decision 1: Tier Switching Strategy

### Option A: Load-and-swap (despawn old, load new)

Current `scene.rs` behavior: despawn existing `ViewerScene` entities immediately, then load the new glTF. During the async load (could be 100ms-2s depending on network), the scene is empty — just camera + lights.

**Pros:** Simple, already implemented for `loadScene`.
**Cons:** Visual gap between scenes. Violates acceptance criteria ("not a blank frame").

### Option B: Crossfade with two scene entities

Keep old scene visible, spawn new scene at opacity 0, fade over 300ms, then despawn old.

**Pros:** Smoothest visual transition.
**Cons:** Requires modifying all materials for opacity (complex in Bevy PBR), doubles GPU memory during transition, doesn't work well with opaque PBR materials. Bevy 0.18 doesn't have built-in scene-level opacity control.

### Option C: Keep-until-ready (deferred despawn)

Keep old scene visible while new scene loads. Once `track_scene_load` confirms the new scene is ready, despawn old and spawn new in the same frame. The swap is a "quick cut" — one frame the old scene is there, next frame the new one.

**Pros:** No blank frames. Simple to implement. No material manipulation needed. GPU memory only doubled briefly (one frame overlap at most). Camera and lights are untouched.
**Cons:** Not a smooth fade. During load, the old tier is still visible (which is fine — user sees the previous tier until the new one is ready).

### Decision: Option C — Keep-until-ready

The acceptance criteria say "fade or quick cut." A quick cut with no blank frames satisfies this cleanly. Crossfade is disproportionately complex for the visual benefit. The key insight: users are switching between tiers of the same project — the geometry is similar. A clean instant swap reads naturally.

Implementation: refactor `SceneState` to track both the old scene entity and the pending new handle. `track_scene_load` spawns the new scene and despawns the old in the same system run.

## Decision 2: SetTier Message Design

### Option A: `setTier` carries a URL

`{ type: "setTier", tier: "better", url: "https://cdn.example.com/better.glb" }` — the host provides both the tier name and the URL. The viewer stores the tier name for bookkeeping, loads the URL.

**Pros:** Viewer stays dumb — doesn't need to know URL patterns. Host has full control over which URLs map to which tiers.
**Cons:** Slightly more data in the message. Host must track URL-to-tier mapping.

### Option B: `setTier` carries only a name

`{ type: "setTier", tier: "better" }` — the viewer maps tier names to URLs using a convention or config.

**Pros:** Simpler message.
**Cons:** Viewer needs to know URL patterns, which ties it to backend conventions. Harder to change URL schemes later.

### Decision: Option A — tier + URL

The viewer is a rendering engine, not a data layer. It should not know about URL conventions. The host owns the mapping. This also means tier toggling works before pt-scene exists — the host can point to any glTF URLs.

The bridge will parse `setTier { tier, url }` into a `SetTierCommand { tier: String, url: String }`. Scene.rs handles the loading. The tier name is stored in `SceneState` so we can report it back if needed.

## Decision 3: Light Angle Feedback

### Option A: Send `lightAngleChanged` on every `setLightAngle` command

Every time the host sends `setLightAngle`, the viewer echoes back `lightAngleChanged { degrees }`. Simple 1:1 response.

**Pros:** Simple. Host always knows current state after sending a command.
**Cons:** The host already knows the angle — it just sent it. Redundant.

### Option B: Send `lightAngleChanged` only on startup (report initial angle)

The viewer sends the initial angle once after setup so the host can display the default time of day.

**Pros:** No redundancy.
**Cons:** Host doesn't have confirmation the command was applied.

### Option C: Echo back on command, plus send initial angle on ready

Covers both: host gets the default angle on startup, and gets confirmation after each change.

### Decision: Option A — echo on every command

The message is tiny (few bytes). The host needs the angle to update its "2:00 PM" display. Even though the host sent the command, the echo confirms the viewer processed it and provides the canonical value. This also handles the startup case: when the viewer sends `ready`, the host can query the angle or the viewer can send the initial angle alongside `ready`.

For the initial angle: we'll send a `lightAngleChanged` in the `setup_lighting` system after spawning the directional light, so the host knows the default angle immediately.

## Decision 4: Scene Transition and Selection State

On tier swap, all `ViewerScene` entities are despawned. The `SelectedZone` resource holds a stale `Entity`. Options:

### Option A: Clear selection on every scene swap

Reset `SelectedZone` to default when a new scene loads. Simple, safe.

### Option B: Restore selection by zone name after new scene loads

Store the selected zone name. After new scene spawns, find the entity with that name and re-select.

### Decision: Option A — clear selection

Re-selecting by name requires waiting for the new scene's entities to be queryable (1+ frames after spawn). This adds complexity. Clearing selection is safe and the UX is reasonable — the user is switching tiers, the previous selection context changes. If a user taps a zone again after switching, it just works.

## Decision 5: Viewer Page UI

The viewer page needs tier toggle buttons and a sunlight slider. Keep it minimal:

- **Tier buttons**: Three buttons (Good / Better / Best), highlight active tier. Call `viewer.setTier(name, url)`.
- **Sunlight slider**: `<input type="range" min="0" max="360">` mapped to degrees. `oninput` calls `viewer.setLightAngle(degrees)`. Display time of day label computed from degrees (0°=6am, 90°=noon, 180°=6pm, 270°=midnight — simplified solar mapping).
- **Light angle display**: Updated from `lightAngleChanged` messages.

Hardcode three test scene URLs for now (all pointing to the same test_scene.glb). When pt-scene exists, the page will get real URLs from the project API.

## Summary of Changes

| Area | Change | Rationale |
|------|--------|-----------|
| `bridge.rs` | Add `SetTierCommand { tier, url }`, add `tier` field to `InboundMessage`, add `send_light_angle_changed()` | Complete the protocol |
| `scene.rs` | Refactor to keep-until-ready: track old entity + new handle, swap on ready. Clear selection via event. | No blank frames |
| `lighting.rs` | Send `lightAngleChanged` after handling command + on initial setup | Host needs angle for time display |
| `types.ts` | Add `lightAngleChanged { degrees }` to `ViewerOutboundMessage`, update type guard | Protocol completeness |
| `Viewer.svelte` | Add `onLightAngleChanged` callback, handle `lightAngleChanged` message | Surface angle to host |
| `+page.svelte` | Add tier toggle buttons + sunlight slider + time display | UI for the features |
| `design.rs` (scenario) | Update S.2.4 to validate `setTier` + `lightAngleChanged` protocol | Test the new protocol additions |
