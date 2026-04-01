# T-014-01 Research: Orbit Camera + Tap-to-Inspect

## Ticket Summary

Add orbit camera controls (mouse + touch), tap-to-inspect with zone highlighting,
camera bounds, and smooth damping to the Bevy WASM viewer.

## Existing Codebase

### Camera (apps/viewer/src/camera.rs)

Uses `bevy_panorbit_camera` 0.34 (`PanOrbitCamera` + `PanOrbitCameraPlugin`).
Current config is minimal:

```rust
PanOrbitCamera {
    button_orbit: MouseButton::Left,
    button_pan: MouseButton::Right,
    zoom_sensitivity: 0.5,
    ..default()
}
```

Missing from current config:
- **No pitch limits** — camera can go underground (negative pitch)
- **No zoom limits** — camera can zoom infinitely close or far
- **No damping** — uses defaults (orbit_smoothness=0.1, pan_smoothness=0.02, zoom_smoothness=0.1)
- **No explicit touch config** — `touch_enabled` defaults to `true`, `touch_controls` defaults to `OneFingerOrbit`
- **No focus bounds** — camera focus can drift anywhere

### bevy_panorbit_camera 0.34 Capabilities

The crate already provides everything we need natively:

| Feature | Field | Default |
|---------|-------|---------|
| Pitch limits | `pitch_upper_limit`, `pitch_lower_limit` | None (unlimited) |
| Zoom limits | `zoom_upper_limit`, `zoom_lower_limit` | None / 0.05 |
| Orbit damping | `orbit_smoothness` | 0.1 |
| Pan damping | `pan_smoothness` | 0.02 |
| Zoom damping | `zoom_smoothness` | 0.1 |
| Touch enable | `touch_enabled` | true |
| Touch scheme | `touch_controls` | OneFingerOrbit |
| Upside-down prevention | `allow_upside_down` | false |

`TouchControls::OneFingerOrbit`: single finger = orbit, two-finger drag = pan, pinch = zoom.
This matches the ticket requirement exactly.

### Picking (apps/viewer/src/picking.rs)

Uses Bevy 0.18 built-in picking: `Pointer<Click>` messages + `Name` component lookup.
On click → looks up entity Name → calls `bridge::send_scene_tapped(name)`.

Missing:
- **No highlight on tap** — tapped mesh has no visual feedback
- **No selection state** — no concept of "currently selected" entity
- **No deselection** — tapping empty space doesn't clear selection

### Bridge (apps/viewer/src/bridge.rs)

Outbound protocol sends `sceneTapped(meshName)`. The ticket specifies
`zoneTapped(zoneId)` — conceptually the same but the message name should change
to match the zone-oriented domain model.

Inbound messages: `loadScene(url)`, `setLightAngle(degrees)`, `setTier(tier)`.
No inbound messages needed for camera/picking — these are local interactions.

### SvelteKit Integration (web/src/lib/components/viewer/)

`Viewer.svelte` listens for `sceneTapped` outbound messages and calls
`onSceneTapped(meshName)`. `types.ts` defines `ViewerOutboundMessage` union type.
Both need updating if we rename to `zoneTapped`.

### Viewer Page (web/src/routes/(app)/project/[id]/viewer/+page.svelte)

Displays tapped mesh name in a div below the viewer. Simple — just needs the
callback name updated.

## Zone-to-Mesh Mapping

Zones don't exist as separate meshes yet (requires pt-scene crate). Currently the
viewer loads a single test glTF. The glTF loader populates `Name` components from
mesh node names, so when pt-scene generates glTF with zone-named meshes, the
picking pipeline will automatically identify zones by name. No special mapping
code is needed — the convention is: glTF mesh name = zone ID.

## Highlight Approach Analysis

### Material-based highlight (emissive color change)
- Bevy `StandardMaterial` has `emissive` and `emissive_intensity` fields
- On tap: query the entity's `MeshMaterial3d<StandardMaterial>`, clone the material,
  set emissive to a highlight color, insert the modified material
- On deselect: restore original material
- Requires storing original material handles per entity

### Outline shader
- Bevy 0.18 has no built-in outline/selection shader
- Third-party crates exist but add binary size and WASM complexity
- Not justified for this ticket

### Color tinting via vertex colors
- Destructive — modifies mesh data
- Not suitable for toggling

**Conclusion**: Material emissive change is the simplest, most reliable approach.
It works with WebGL2, requires no additional dependencies, and is easily reversible.

## Performance Constraints

- Target: 60 FPS desktop, 30+ FPS iPad
- Current binary: ~10 MB WASM (well under 15 MB budget)
- Material clone per tap is negligible cost (one allocation)
- PanOrbitCamera smoothing uses lerp per frame — zero allocation, ~3 multiplies

## Key Constraints

1. bevy_panorbit_camera handles all orbit/pan/zoom/touch/damping natively
2. Highlight must work with WebGL2 (no compute shaders, no post-processing that requires MRT)
3. Zone IDs = glTF mesh names (convention, not code dependency)
4. No new crate dependencies needed — Bevy + bevy_panorbit_camera cover everything
5. The "touch" feature in Bevy's Cargo.toml is already enabled in apps/viewer/Cargo.toml
