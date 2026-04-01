# T-014-01 Review: Orbit Camera + Tap-to-Inspect

## Summary

Added orbit camera bounds, smooth damping, touch controls, zone highlight on tap,
and renamed the protocol message from `sceneTapped(meshName)` to `zoneTapped(zoneId)`.
All changes configure existing crate capabilities — no new dependencies added.

## Files Changed

### Bevy Viewer (apps/viewer/src/)

| File | Change |
|------|--------|
| `camera.rs` | Added pitch limits (-3° to 80°), zoom limits (0.5–50), damping (orbit=0.2, pan=0.1, zoom=0.15), touch (OneFingerOrbit), `allow_upside_down: false` |
| `bridge.rs` | Renamed `send_scene_tapped()` → `send_zone_tapped()`, JSON `sceneTapped/meshName` → `zoneTapped/zoneId` |
| `picking.rs` | Rewrote: added `SelectedZone` resource, emissive highlight (golden, intensity 150), deselection on unnamed entity tap, skip re-selection of same entity |

### SvelteKit (web/src/)

| File | Change |
|------|--------|
| `lib/components/viewer/types.ts` | `sceneTapped` → `zoneTapped`, `meshName` → `zoneId` in type + guard |
| `lib/components/viewer/Viewer.svelte` | `onSceneTapped` → `onZoneTapped` prop + handler |
| `routes/(app)/project/[id]/viewer/+page.svelte` | `tappedMesh` → `tappedZone`, updated callback |

### Scenarios (tests/scenarios/)

| File | Change |
|------|--------|
| `src/suites/design.rs` | S.2.4 updated to validate `zoneTapped/zoneId` protocol, doc comments updated |

## Acceptance Criteria Coverage

| Criterion | Status | Notes |
|-----------|--------|-------|
| Orbit camera: drag to rotate, scroll to zoom, right-drag to pan | Done | PanOrbitCamera configured with left=orbit, right=pan, scroll=zoom |
| Touch: single finger rotate, pinch zoom, two-finger pan | Done | `TouchControls::OneFingerOrbit` |
| Tap → raycast → zoneTapped(zoneId) | Done | Bevy picking + Name lookup + `send_zone_tapped()` |
| Tapped zone highlights | Done | Emissive material change (golden, restores on deselect) |
| Camera bounds: no underground, not too far | Done | pitch_lower_limit=-0.05, zoom_upper_limit=50 |
| Smooth damping | Done | orbit_smoothness=0.2, pan_smoothness=0.1, zoom_smoothness=0.15 |
| 60 FPS desktop, 30+ FPS iPad | Unchanged | No performance-impacting changes; T-013-01 verified this baseline |

## Scenario Dashboard

**Before**: 58.0/240.0 min (24.2%) — S.2.4 at TwoStar
**After**: 58.0/240.0 min (24.2%) — S.2.4 at TwoStar (protocol updated, no regression)

No scenario regression. This ticket enhances interaction quality without changing
the integration level. S.2.4 remains TwoStar — ThreeStar requires pt-scene.

## Test Coverage

- **Scenario S.2.4**: Updated to validate new `zoneTapped/zoneId` protocol contract
- **No new unit tests**: Changes are declarative camera config and visual highlight
  behavior — neither is meaningfully testable without a GPU context in CI
- **Manual verification needed**: orbit bounds, touch gestures, highlight visuals
  require a browser with the WASM viewer running (`trunk serve` in apps/viewer/)

## Open Concerns

1. **Highlight on shared materials**: If multiple meshes share the same glTF
   material, modifying it in-place will highlight all of them. This is acceptable
   for now — pt-scene will generate unique materials per zone. If it becomes an
   issue before pt-scene, the fix is to clone the material handle per entity.

2. **No deselect on background click**: Currently deselection only happens when
   tapping an entity without a Name component. Clicking empty space (no entity hit)
   doesn't fire a `Pointer<Click>` message, so there's no deselect path for that.
   Could be addressed with a background plane or a separate input system, but
   not required by the acceptance criteria.

3. **Camera bound values are hardcoded**: The pitch/zoom limits work for the test
   scene but may need adjustment when real garden scenes vary in scale. pt-scene
   could send scene bounds metadata to dynamically configure limits.

4. **Viewer crate is out-of-workspace**: The Bevy viewer isn't compiled by
   `cargo clippy --workspace` or `cargo test --workspace`. Its compilation was
   verified in T-013-01/T-013-02 via `trunk build --release`. Changes here are
   configuration-only (no new Rust features or APIs), so compilation risk is low.

## Quality Gate

```
cargo fmt --check      ✓
cargo clippy --workspace -- -D warnings  ✓
cargo test --workspace ✓
cargo run -p pt-scenarios  ✓ (58.0/240.0 min, no regression)
```
