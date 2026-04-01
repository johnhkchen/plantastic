# T-014-01 Structure: Orbit Camera + Tap-to-Inspect

## Files Modified

### apps/viewer/src/camera.rs

**Changes**: Expand `PanOrbitCamera` configuration with bounds, damping, and touch.

```
PanOrbitCamera {
    button_orbit: MouseButton::Left,
    button_pan: MouseButton::Right,
    zoom_sensitivity: 0.5,
+   // Pitch bounds — prevent underground and gimbal lock
+   pitch_upper_limit: Some(~80°),
+   pitch_lower_limit: Some(~-3°),
+   allow_upside_down: false,
+   // Zoom bounds
+   zoom_upper_limit: Some(50.0),
+   zoom_lower_limit: 0.5,
+   // Damping
+   orbit_smoothness: 0.2,
+   pan_smoothness: 0.1,
+   zoom_smoothness: 0.15,
+   // Touch
+   touch_enabled: true,
+   touch_controls: TouchControls::OneFingerOrbit,
}
```

Import added: `TouchControls` from `bevy_panorbit_camera`.

### apps/viewer/src/picking.rs

**Changes**: Add selection state, highlight logic, deselection.

New resource:
```rust
#[derive(Resource, Default)]
struct SelectedZone {
    entity: Option<Entity>,
    original_material: Option<Handle<StandardMaterial>>,
}
```

Modified system `handle_click_messages`:
- On tap with Name: highlight entity, store selection, send zoneTapped
- On tap without Name: clear selection, restore material

New system `apply_highlight`:
- Queries MeshMaterial3d<StandardMaterial> on selected entity
- Clones material, sets emissive color, inserts modified material
- Restores previous selection's original material

New function `create_highlight_material`:
- Takes &StandardMaterial → StandardMaterial with emissive boost

Import additions: `StandardMaterial`, `MeshMaterial3d`, `Assets`.

### apps/viewer/src/bridge.rs

**Changes**: Rename outbound message.

```diff
-pub fn send_scene_tapped(mesh_name: &str) {
-    let json = json!({ "type": "sceneTapped", "meshName": mesh_name });
+pub fn send_zone_tapped(zone_id: &str) {
+    let json = json!({ "type": "zoneTapped", "zoneId": zone_id });
```

### web/src/lib/components/viewer/types.ts

**Changes**: Update outbound message type.

```diff
 export type ViewerOutboundMessage =
     | { type: 'ready' }
     | { type: 'error'; message: string }
-    | { type: 'sceneTapped'; meshName: string };
+    | { type: 'zoneTapped'; zoneId: string };
```

Update `isViewerMessage()` type guard: `sceneTapped` → `zoneTapped`.

### web/src/lib/components/viewer/Viewer.svelte

**Changes**: Update callback prop and message handler.

```diff
-onSceneTapped?: (meshName: string) => void;
+onZoneTapped?: (zoneId: string) => void;
```

Message handler case: `zoneTapped` → `onZoneTapped(msg.zoneId)`.

### web/src/routes/(app)/project/[id]/viewer/+page.svelte

**Changes**: Update callback usage.

```diff
-onSceneTapped={(name) => tappedMesh = name}
+onZoneTapped={(id) => tappedZone = id}
```

### tests/scenarios/src/suites/design.rs

**Changes**: Update S.2.4 protocol test to use `zoneTapped` instead of `sceneTapped`.

```diff
-let tapped = json!({ "type": "sceneTapped", "meshName": "patio_travertine" });
+let tapped = json!({ "type": "zoneTapped", "zoneId": "patio_travertine" });
```

Update assertion field name and type guard check.

## Files NOT Modified

- `apps/viewer/src/main.rs` — no new plugins, just configuration changes
- `apps/viewer/src/scene.rs` — no changes to scene loading
- `apps/viewer/src/lighting.rs` — no changes to lighting
- `apps/viewer/Cargo.toml` — no new dependencies needed

## Module Boundaries

- `picking.rs` owns selection state and highlight logic
- `bridge.rs` owns message protocol (picking calls bridge outbound helpers)
- `camera.rs` is self-contained configuration, no interaction with other modules
- SvelteKit types.ts is the TypeScript source of truth for the message protocol
