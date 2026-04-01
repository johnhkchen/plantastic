# T-014-01 Plan: Orbit Camera + Tap-to-Inspect

## Step 1: Camera configuration

**File**: `apps/viewer/src/camera.rs`

Update `PanOrbitCamera` spawn with:
- Pitch limits (prevent underground + gimbal lock)
- Zoom limits (min/max distance)
- Damping values (orbit, pan, zoom smoothness)
- Touch controls (enabled, OneFingerOrbit)
- `allow_upside_down: false`

**Verify**: Compiles. No runtime test possible without WASM browser, but the
configuration is declarative — if it compiles, it's correct.

## Step 2: Rename bridge outbound message

**File**: `apps/viewer/src/bridge.rs`

- Rename `send_scene_tapped()` → `send_zone_tapped()`
- Change parameter name `mesh_name` → `zone_id`
- Update JSON: `"type": "zoneTapped"`, `"zoneId": zone_id`

**Verify**: Compiles. Caller in picking.rs will need updating (step 3).

## Step 3: Add selection state and highlight to picking

**File**: `apps/viewer/src/picking.rs`

- Add `SelectedZone` resource (entity + original material handle)
- Register resource in plugin build
- Modify `handle_click_messages`:
  - On click with Name: deselect previous, select new, send zoneTapped
  - On click without Name: deselect previous
- Add highlight system that applies/removes emissive material changes
- Call `bridge::send_zone_tapped()` instead of `send_scene_tapped()`

**Verify**: Compiles. The highlight behavior is visual — verified by the existing
manual test workflow (Trunk serve → click meshes in browser).

## Step 4: Update SvelteKit types

**File**: `web/src/lib/components/viewer/types.ts`

- Change `sceneTapped` → `zoneTapped` in `ViewerOutboundMessage`
- Change `meshName` → `zoneId`
- Update `isViewerMessage()` type guard

**Verify**: TypeScript compilation via `npm run check` in web/.

## Step 5: Update Viewer component

**File**: `web/src/lib/components/viewer/Viewer.svelte`

- Rename prop `onSceneTapped` → `onZoneTapped`
- Update message handler case from `sceneTapped` to `zoneTapped`
- Update callback invocation to pass `msg.zoneId`

**Verify**: TypeScript compilation.

## Step 6: Update viewer page

**File**: `web/src/routes/(app)/project/[id]/viewer/+page.svelte`

- Update callback prop name and variable name

**Verify**: TypeScript compilation.

## Step 7: Update scenario test

**File**: `tests/scenarios/src/suites/design.rs`

- Update S.2.4 to validate `zoneTapped` / `zoneId` instead of `sceneTapped` / `meshName`

**Verify**: `cargo test -p pt-scenarios` passes. `cargo run -p pt-scenarios` shows
S.2.4 still at TwoStar (no regression).

## Step 8: Run quality gate

Run `just check` (fmt + lint + test + scenarios).

**Verify**: All four checks pass. Scenario dashboard shows no regressions.

## Testing Strategy

- **Unit tests**: None added — all changes are either declarative configuration
  (camera fields) or visual behavior (highlight) that can't be meaningfully unit
  tested without a GPU context.
- **Scenario test**: S.2.4 updated to validate the renamed protocol.
- **Manual verification**: Trunk serve the viewer, test orbit/pan/zoom/touch/tap
  in browser. This is documented in progress.md.
- **Integration**: The protocol contract test in S.2.4 ensures the Bevy bridge
  and SvelteKit types agree on message shapes.
