# T-014-01 Design: Orbit Camera + Tap-to-Inspect

## Decisions

### 1. Camera Controls: Configure bevy_panorbit_camera (not custom)

**Decision**: Configure existing `PanOrbitCamera` fields rather than writing custom
camera logic.

**Why**: bevy_panorbit_camera 0.34 already handles orbit, pan, zoom, touch gestures,
damping, and bounds. Writing custom input handling would duplicate proven code and
introduce bugs (especially touch gesture math). The crate is already a dependency.

**Rejected**: Custom orbit camera system. Cost: ~300 lines of quaternion math, touch
gesture detection, edge-case handling (gimbal lock, upside-down). Benefit: none —
the crate does all of this.

### 2. Camera Bounds: Pitch limits + zoom limits

**Configuration**:
```
pitch_upper_limit: Some(PI/2 - 0.1)     ~80° (nearly overhead, not quite gimbal lock)
pitch_lower_limit: Some(-0.05)           ~-3° (just below horizon, prevents underground)
zoom_upper_limit: Some(50.0)             Don't fly away
zoom_lower_limit: 0.5                    Don't clip through model
allow_upside_down: false                 Prevent camera inversion
```

**Why these values**: A landscaper orbiting a garden design needs to see from ground
level to near-overhead. The slight negative pitch (-3°) allows a natural ground-level
view without going underground. Zoom limits keep the model visible and prevent
accidental zoom-to-infinity.

**Rejected**: `focus_bounds_shape` (Sphere/Cuboid restriction on the focus point).
Unnecessary — the focus point starts at origin and the scene is centered. Adding
bounds now would be premature; the pt-scene crate may need to set bounds dynamically
based on garden dimensions.

### 3. Damping: Moderate smoothing

**Configuration**:
```
orbit_smoothness: 0.2     Noticeably smooth, not sluggish
pan_smoothness: 0.1       Pan needs to feel responsive
zoom_smoothness: 0.15     Smooth scroll zoom
```

**Why**: Defaults (orbit=0.1, pan=0.02) feel slightly twitchy. The ticket requires
"smooth damping on camera movement (not snappy)." These values provide visible
smoothing without the camera feeling laggy. Tested range: 0.0 (instant) to 0.5
(noticeably delayed). Values above 0.3 feel unresponsive on touch devices.

### 4. Touch Controls: OneFingerOrbit mode

**Configuration**:
```
touch_enabled: true
touch_controls: TouchControls::OneFingerOrbit
```

**Mapping**:
- Single finger → orbit (rotate around focus)
- Two-finger drag → pan
- Pinch → zoom

**Why**: This matches the ticket requirement exactly ("single finger to rotate,
pinch to zoom, two-finger drag to pan"). The alternative `TwoFingerOrbit` swaps
orbit and pan — less intuitive for a 3D model viewer where the primary action is
rotating to see different angles.

### 5. Highlight: Emissive material swap

**Decision**: On tap, clone the entity's `StandardMaterial`, boost `emissive` to a
highlight color (golden-yellow), and insert it. On deselect, restore the original.

**Implementation**:
- `SelectedZone` resource tracks: entity, original material handle
- On tap: save current material, insert highlighted clone
- On tap of different mesh: restore previous, highlight new
- On tap of empty space (no Name component): restore previous, clear selection

**Highlight color**: `Color::srgb(1.0, 0.8, 0.2)` (warm golden) with
`emissive_intensity: 2.0`. Visible on any base material without obscuring texture.

**Rejected alternatives**:
- **Outline shader**: No built-in support in Bevy 0.18 for WebGL2. Third-party
  crates add binary size and complexity. Not worth it for an MVP highlight.
- **Vertex color tinting**: Destructive to mesh data, complex to reverse.
- **Wireframe overlay**: Bevy wireframe plugin exists but is a global toggle, not
  per-entity. Not suitable for selection highlight.

### 6. Message Protocol: Rename sceneTapped → zoneTapped

**Decision**: Rename the outbound message from `sceneTapped(meshName)` to
`zoneTapped(zoneId)` on both Bevy and SvelteKit sides.

**Why**: The ticket explicitly specifies `zoneTapped(zoneId)`. The domain model is
zone-centric — landscapers tap zones, not arbitrary meshes. The field name `zoneId`
(rather than `meshName`) aligns with the API model where zones have IDs that match
glTF mesh names.

**Scope**: This is a rename, not a new message. Update:
- `bridge.rs`: `send_scene_tapped()` → `send_zone_tapped()`, JSON field `zoneId`
- `types.ts`: `sceneTapped` → `zoneTapped`, `meshName` → `zoneId`
- `Viewer.svelte`: `onSceneTapped` → `onZoneTapped` callback
- `isViewerMessage()`: update type guard
- Viewer page: update callback usage

### 7. Scenario Impact

No new scenario — S.2.4 already covers the viewer pipeline at TwoStar. This ticket
enhances the interaction quality but doesn't change the integration level. The
scenario test validates the protocol contract, so we update it to use `zoneTapped`
instead of `sceneTapped`.

The milestone "Bevy viewer: glTF loading + orbit + tap-to-inspect" (delivered by
T-013-02) already covers this area. No new milestone needed.
