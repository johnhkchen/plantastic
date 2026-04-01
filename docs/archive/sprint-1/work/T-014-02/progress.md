# T-014-02 Progress — Tier Toggle + Sunlight Control

## Completed Steps

### Step 1: Extend bridge protocol (bridge.rs)
- Added `SetTierCommand { tier, url }` message type
- Added `tier` field to `InboundMessage` deserialization
- Registered `SetTierCommand` in `BridgePlugin::build`
- Added `"setTier"` case to `drain_messages` (requires both tier and url)
- Added `send_light_angle_changed()` and `send_tier_changed()` outbound helpers

### Step 2: Refactor scene loading (scene.rs)
- Refactored `SceneState` to track pending vs current: `pending_handle`, `pending_tier`, `current_tier`
- Added `handle_set_tier` system that stores pending handle without despawning old scene
- Modified `track_scene_load` for keep-until-ready: old scene stays visible until new glTF is loaded
- On swap: despawn old, spawn new, clear `SelectedZone`, send `tierChanged` + `ready`
- `handle_load_scene` also uses keep-until-ready path

### Step 3: Add light angle feedback (lighting.rs)
- Added `bridge::send_light_angle_changed(cmd.degrees)` after transform update in `handle_light_angle`
- Added `bridge::send_light_angle_changed(30.0)` in `setup_lighting` for initial angle

### Step 3b: Fix pre-existing picking.rs compilation errors (rule 6)
- Removed `emissive_intensity` field references (not in Bevy 0.18 `StandardMaterial`)
- Fixed borrow conflict by cloning material before mutating
- Made `SelectedZone` public so `scene.rs` can reset it on tier swap

### Step 4: Viewer compiles
- `cargo check` passes in `apps/viewer/`

### Step 5: Extend TypeScript types (types.ts)
- Added `lightAngleChanged { degrees }` and `tierChanged { tier }` to `ViewerOutboundMessage`
- Updated `setTier` inbound to include `url` field
- Updated `isViewerMessage` type guard

### Step 6: Update Viewer.svelte
- Added `onLightAngleChanged` and `onTierChanged` callback props
- Updated `setTier(tier, url)` signature to include URL
- Added message handler cases for new outbound types

### Step 7: Add UI controls to viewer page
- Added tier toggle button group (Good / Better / Best) with active state highlight
- Added sunlight slider (0-360°) with time-of-day display
- Added `degreesToTime()` mapping (0°=6am, 90°=noon, 180°=6pm)
- Wired callbacks for light angle and tier change feedback

### Step 8: Update scenario test (design.rs)
- Extended S.2.4 to validate `setTier { tier, url }` (url now required)
- Added `lightAngleChanged { degrees }` outbound validation
- Added `tierChanged { tier }` outbound validation
- Updated docstring with T-014-02 contributions
- Result remains TwoStar

### Step 9: Quality gate
- `just check` passes: fmt, lint, test, scenarios all green
- No regressions in any scenario

## Deviations from Plan

- **Picking.rs fix**: Pre-existing compilation errors in picking.rs (Bevy 0.18 removed `emissive_intensity`). Fixed per CLAUDE.md rule 6 ("Own what you find"). This was not in the original plan.
- **SelectedZone visibility**: Made `SelectedZone` struct public to allow scene.rs to clear it. This was implied in the structure doc but not explicitly called out.
