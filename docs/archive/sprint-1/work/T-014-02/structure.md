# T-014-02 Structure — File-Level Changes

## Bevy Viewer (`apps/viewer/src/`)

### `bridge.rs` — Modified

**Add `SetTierCommand` message:**
```rust
#[derive(Message)]
pub struct SetTierCommand {
    pub tier: String,
    pub url: String,
}
```

**Extend `InboundMessage` deserialization:**
```rust
#[derive(Deserialize)]
struct InboundMessage {
    #[serde(rename = "type")]
    msg_type: String,
    url: Option<String>,
    degrees: Option<f32>,
    tier: Option<String>,  // NEW
}
```

**Add `SetTierCommand` writer to `drain_messages`:**
Handle `"setTier"` case — extract `tier` and `url`, write `SetTierCommand`.

**Register `SetTierCommand` in plugin build:**
```rust
app.add_message::<SetTierCommand>()
```

**Add outbound helper:**
```rust
pub fn send_light_angle_changed(degrees: f32) {
    let json = serde_json::json!({ "type": "lightAngleChanged", "degrees": degrees }).to_string();
    post_to_parent(&json);
}

pub fn send_tier_changed(tier: &str) {
    let json = serde_json::json!({ "type": "tierChanged", "tier": tier }).to_string();
    post_to_parent(&json);
}
```

### `scene.rs` — Modified

**Refactor `SceneState` for keep-until-ready:**
```rust
#[derive(Resource, Default)]
struct SceneState {
    pending_handle: Option<Handle<Gltf>>,
    pending_tier: Option<String>,
    current_tier: Option<String>,
    spawned: bool,
}
```

**Add `SetTierCommand` handler:**
New system `handle_set_tier` reads `SetTierCommand` messages. Stores the pending handle and tier name. Does NOT despawn the old scene yet.

**Modify `track_scene_load`:**
When the pending glTF becomes available:
1. Despawn all `ViewerScene` entities (old scene)
2. Spawn new `SceneRoot` with `ViewerScene` marker
3. Update `current_tier`
4. Clear `SelectedZone` resource (reset to default)
5. Send `tierChanged` back to host
6. Send `ready`

**Keep `handle_load_scene` for direct `loadScene` commands:**
The initial `loadScene` from host still works as before, but also goes through the keep-until-ready path.

### `lighting.rs` — Modified

**Send `lightAngleChanged` after handling command:**
At the end of `handle_light_angle`, call `bridge::send_light_angle_changed(cmd.degrees)`.

**Send initial angle on setup:**
In `setup_lighting`, after spawning the directional light, call `bridge::send_light_angle_changed(30.0)` (the default yaw in degrees — note: initial yaw is π/6 radians = 30°).

### `camera.rs` — No changes

Camera is a separate entity, not part of `ViewerScene`. Survives scene swaps naturally.

### `picking.rs` — No changes

Selection clearing is handled by `scene.rs` resetting the `SelectedZone` resource. The `handle_click_messages` system already handles `SelectedZone::default()` gracefully (no entity selected = nothing to deselect).

### `main.rs` — No changes

All new systems are registered via existing plugins.

---

## SvelteKit Host (`web/src/`)

### `lib/components/viewer/types.ts` — Modified

**Extend `ViewerOutboundMessage`:**
```typescript
export type ViewerOutboundMessage =
    | { type: 'ready' }
    | { type: 'error'; message: string }
    | { type: 'zoneTapped'; zoneId: string }
    | { type: 'lightAngleChanged'; degrees: number }   // NEW
    | { type: 'tierChanged'; tier: string };             // NEW
```

**Update `isViewerMessage` type guard:**
Add `'lightAngleChanged'` and `'tierChanged'` to the valid type checks.

### `lib/components/viewer/Viewer.svelte` — Modified

**Add new callback props:**
```typescript
onLightAngleChanged?: (degrees: number) => void;
onTierChanged?: (tier: string) => void;
```

**Handle new outbound messages in `$effect`:**
```typescript
case 'lightAngleChanged':
    onLightAngleChanged?.(msg.degrees);
    break;
case 'tierChanged':
    onTierChanged?.(msg.tier);
    break;
```

### `routes/(app)/project/[id]/viewer/+page.svelte` — Modified

**Add state:**
```typescript
let activeTier = $state('good');
let lightAngle = $state(30);
let viewerRef: Viewer;
```

**Add tier toggle buttons:**
Three buttons (Good / Better / Best) in a button group. `onclick` calls `viewerRef.setTier(tier)` and updates `activeTier`.

**Add sunlight slider:**
Range input 0-360. `oninput` calls `viewerRef.setLightAngle(value)`. Label shows approximate time of day computed from degrees.

**Wire callbacks:**
- `onLightAngleChanged` → update `lightAngle` state → update time display
- `onTierChanged` → update `activeTier` state → highlight active button

---

## Scenario Tests (`tests/scenarios/src/suites/design.rs`)

### `s_2_4_3d_preview` — Modified

Add validation for the two new protocol additions:
1. `setTier` inbound message now includes `url` field
2. `lightAngleChanged` outbound message with `degrees` field
3. `tierChanged` outbound message with `tier` field

Result remains TwoStar — this ticket adds protocol completeness but doesn't change the integration level.

---

## Module Boundaries

```
bridge.rs  ←→  scene.rs      (SetTierCommand message)
bridge.rs  ←→  lighting.rs   (SetLightAngleCommand message, send_light_angle_changed fn)
scene.rs   →   picking.rs    (clears SelectedZone resource on scene swap)
```

No new crate dependencies. No new files created. All changes are modifications to existing files.

## Change Ordering

1. `bridge.rs` first — defines the new message types and outbound helpers
2. `scene.rs` second — depends on `SetTierCommand` from bridge
3. `lighting.rs` third — depends on `send_light_angle_changed` from bridge
4. `types.ts` fourth — extends protocol types
5. `Viewer.svelte` fifth — depends on types.ts changes
6. `+page.svelte` sixth — depends on Viewer.svelte changes
7. `design.rs` last — scenario test updates
