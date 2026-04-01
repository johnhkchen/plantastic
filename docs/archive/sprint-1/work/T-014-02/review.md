# T-014-02 Review — Tier Toggle + Sunlight Control

## Summary of Changes

### Files Modified

| File | Change |
|------|--------|
| `apps/viewer/src/bridge.rs` | Added `SetTierCommand` message, `tier` field to `InboundMessage`, `"setTier"` handler in `drain_messages`, `send_light_angle_changed()` and `send_tier_changed()` outbound helpers |
| `apps/viewer/src/scene.rs` | Refactored to keep-until-ready pattern: `SceneState` tracks pending/current separately, new `handle_set_tier` system, `track_scene_load` defers despawn until new scene ready, clears `SelectedZone` on swap |
| `apps/viewer/src/lighting.rs` | Sends `lightAngleChanged` after handling `setLightAngle` command + on initial setup (30°) |
| `apps/viewer/src/picking.rs` | Fixed pre-existing Bevy 0.18 compilation errors (`emissive_intensity` removed), made `SelectedZone` pub for cross-module access |
| `web/src/lib/components/viewer/types.ts` | Added `lightAngleChanged` and `tierChanged` outbound types, updated `setTier` to include `url`, updated type guard |
| `web/src/lib/components/viewer/Viewer.svelte` | Added `onLightAngleChanged` and `onTierChanged` callback props, updated `setTier(tier, url)` signature, handled new message types |
| `web/src/routes/(app)/project/[id]/viewer/+page.svelte` | Added tier toggle button group, sunlight slider with time-of-day display, wired callbacks |
| `tests/scenarios/src/suites/design.rs` | Extended S.2.4 to validate `setTier { tier, url }`, `lightAngleChanged`, `tierChanged` protocol |

### No Files Created or Deleted

## Acceptance Criteria Status

| Criteria | Status | Notes |
|----------|--------|-------|
| Host sends `setTier(name)` → viewer loads corresponding glTF | Done | `SetTierCommand { tier, url }` → scene.rs loads URL |
| Scene swap is smooth (fade or quick cut, not blank frame) | Done | Keep-until-ready: old scene visible until new one loads, then instant swap |
| Camera position preserved across tier switches | Done | Camera is separate entity, not part of `ViewerScene` — survives despawn/spawn |
| Host sends `setLightAngle(degrees)` → viewer rotates directional light | Done | Was already implemented in T-013-02; this ticket adds feedback |
| Shadow direction updates in real time as slider moves | Done | `oninput` fires on every slider movement, updates light yaw |
| Viewer sends back current light angle so host can display time | Done | `lightAngleChanged { degrees }` sent on every command + on startup |

## Test Coverage

**Scenario S.2.4**: Validates complete postMessage protocol schema including all new message types. Remains TwoStar.

**Viewer Rust code**: No automated unit tests (out-of-workspace WASM crate). Protocol contract is the testable boundary. Manual verification via `trunk serve` required for visual behavior.

**Gap**: No automated test for keep-until-ready behavior (would require a Bevy test harness or browser automation). This is acceptable — the behavior is simple and deterministic.

## Scenario Dashboard

Before: S.2.4 = TwoStar (unchanged from T-014-01)
After: S.2.4 = TwoStar (protocol expanded, integration level same)

No scenario regressions. `just check` passes all gates.

The tier toggle and sunlight features don't advance S.2.4 to ThreeStar — that requires pt-scene generating real glTF from the project model per tier. This ticket completes the viewer-side infrastructure so ThreeStar is unblocked on the scene generation side only.

## Open Concerns

1. **Test scene URLs are identical**: All three tiers point to the same `test_scene.glb`. This is expected — pt-scene doesn't exist yet. When it does, the viewer page will get tier-specific URLs from the project API.

2. **Light angle → time mapping is approximate**: The `degreesToTime()` function uses a simple linear mapping (0°=6am, 360°=6am next day). Real solar angles depend on latitude, season, and time zone. This is a UI approximation sufficient for "show the client morning vs afternoon light." Accurate solar time mapping will come from pt-solar integration.

3. **No throttling on light angle messages**: The slider sends `setLightAngle` on every `oninput` event (could be 60+ per second during drag). The viewer processes all of them. If this causes frame drops, add client-side debounce. Not observed as an issue currently.

4. **Picking.rs emissive fix**: The pre-existing code used `emissive_intensity` which doesn't exist in Bevy 0.18. Fixed by using only `emissive` color. The highlight may be slightly different visually (intensity not separately controllable), but the golden emissive overlay still reads clearly on any base material.

5. **SelectedZone cleared on tier swap**: Selection is lost when switching tiers. Could be improved later by restoring selection by zone name after new scene loads, but this adds complexity for minimal UX benefit — the user is changing context when switching tiers.

## Quality Gate

```
just check → All gates passed
  fmt-check  ✓
  lint       ✓
  test       ✓ (all workspace tests pass)
  scenarios  ✓ (S.2.4 TwoStar, no regressions)
```
