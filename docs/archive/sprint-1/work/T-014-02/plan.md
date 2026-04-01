# T-014-02 Plan — Implementation Steps

## Step 1: Extend bridge protocol (bridge.rs)

- Add `SetTierCommand { tier: String, url: String }` message type
- Add `tier: Option<String>` field to `InboundMessage`
- Register `SetTierCommand` in `BridgePlugin::build`
- Add `SetTierCommand` writer param to `drain_messages`, handle `"setTier"` case
- Add `send_light_angle_changed(degrees: f32)` outbound helper
- Add `send_tier_changed(tier: &str)` outbound helper

**Verify:** `cargo check -p plantastic-viewer` (or equivalent — viewer is out-of-workspace, so `cd apps/viewer && cargo check`)

## Step 2: Refactor scene loading for keep-until-ready (scene.rs)

- Refactor `SceneState` to track pending handle/tier separately from current
- Add `handle_set_tier` system that reads `SetTierCommand`, stores pending handle without despawning old scene
- Modify `track_scene_load`: when pending glTF is ready, despawn old `ViewerScene` entities, spawn new, clear `SelectedZone`, send `tierChanged` + `ready`
- Unify `handle_load_scene` to also use the keep-until-ready path (not just `setTier`)

**Verify:** `cargo check` in apps/viewer/

## Step 3: Add light angle feedback (lighting.rs)

- In `handle_light_angle`: after updating transform, call `bridge::send_light_angle_changed(cmd.degrees)`
- In `setup_lighting`: after spawning directional light, call `bridge::send_light_angle_changed(30.0)` to report initial angle

**Verify:** `cargo check` in apps/viewer/

## Step 4: Build WASM and verify (apps/viewer/)

- `cd apps/viewer && trunk build --release`
- Verify binary builds cleanly for `wasm32-unknown-unknown`

## Step 5: Extend TypeScript protocol types (types.ts)

- Add `lightAngleChanged` and `tierChanged` to `ViewerOutboundMessage`
- Update `isViewerMessage` type guard to include new types

**Verify:** TypeScript compiles (no separate step needed — checked with page changes)

## Step 6: Update Viewer.svelte bridge component

- Add `onLightAngleChanged` and `onTierChanged` callback props
- Handle `lightAngleChanged` and `tierChanged` cases in message handler switch

**Verify:** No runtime errors in component

## Step 7: Add tier toggle + sunlight slider to viewer page (+page.svelte)

- Add `activeTier`, `lightAngle`, `viewerRef` state variables
- Add tier toggle button group (Good / Better / Best)
- Add sunlight slider (range 0-360) with time-of-day label
- Wire `onLightAngleChanged` and `onTierChanged` callbacks
- Use test scene URLs for all three tiers (same glb for now)

**Verify:** Page renders with controls

## Step 8: Update scenario test (design.rs)

- Extend S.2.4 to validate `setTier { tier, url }` inbound schema
- Validate `lightAngleChanged { degrees }` outbound schema
- Validate `tierChanged { tier }` outbound schema
- Keep result at TwoStar

**Verify:** `just scenarios` — S.2.4 still passes, no regressions

## Step 9: Run quality gate

- `just check` (fmt + lint + test + scenarios)
- Fix any issues

## Testing Strategy

**Unit tests:** The viewer is out-of-workspace WASM — no unit tests in the workspace test suite. Protocol correctness is validated in the scenario test.

**Scenario test (S.2.4):** Validate the complete postMessage protocol schema including new message types. This is the primary automated verification.

**Manual verification:** `trunk serve --release` in apps/viewer/, load in browser, test:
- Tier switching preserves camera position
- No blank frame on switch (keep-until-ready)
- Sunlight slider moves shadows in real time
- Console shows `lightAngleChanged` messages

## Commit Strategy

1. After Step 3: commit Bevy-side changes (bridge + scene + lighting)
2. After Step 7: commit SvelteKit-side changes (types + Viewer + page)
3. After Step 8: commit scenario test update
