# T-014-01 Progress: Orbit Camera + Tap-to-Inspect

## Completed Steps

### Step 1: Camera configuration ✓
- Updated `apps/viewer/src/camera.rs` with pitch limits, zoom limits, damping, and touch controls
- Added `TouchControls` import from `bevy_panorbit_camera`

### Step 2: Rename bridge outbound message ✓
- Renamed `send_scene_tapped()` → `send_zone_tapped()` in `apps/viewer/src/bridge.rs`
- Updated JSON output: `"type": "zoneTapped"`, `"zoneId": zone_id`

### Step 3: Selection state and highlight ✓
- Rewrote `apps/viewer/src/picking.rs` with `SelectedZone` resource
- Added emissive material highlight on tap (golden-yellow, intensity 150)
- Added deselection on tap of unnamed entity
- Skip if same entity tapped again

### Step 4: SvelteKit types ✓
- Updated `web/src/lib/components/viewer/types.ts`: `sceneTapped` → `zoneTapped`, `meshName` → `zoneId`
- Updated `isViewerMessage()` type guard

### Step 5: Viewer component ✓
- Updated `web/src/lib/components/viewer/Viewer.svelte`: `onSceneTapped` → `onZoneTapped`
- Updated message handler case

### Step 6: Viewer page ✓
- Updated `web/src/routes/(app)/project/[id]/viewer/+page.svelte`
- Renamed variable `tappedMesh` → `tappedZone`

### Step 7: Scenario test ✓
- Updated `tests/scenarios/src/suites/design.rs` S.2.4 to use `zoneTapped` / `zoneId`
- Updated doc comments to reference T-014-01 changes

### Step 8: Quality gate ✓
- `cargo fmt --check` — pass
- `cargo clippy --workspace -- -D warnings` — pass
- `cargo test --workspace` — pass
- `cargo run -p pt-scenarios` — 58.0/240.0 min (24.2%), no regressions

## Deviations from Plan

None. All steps executed as planned.
