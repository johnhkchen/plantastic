# T-013-01 Design — Bevy WASM glTF Loading

## Decision summary

Build a standalone Bevy 0.15 app in `apps/viewer/` using Trunk for WASM compilation.
Target WebGL2 for maximum browser compatibility. Use the Khronos DamagedHelmet.glb as
the test model. Aggressively gate Bevy features to minimize binary size. Include orbit
camera and FPS overlay for verification. Do NOT add to the Cargo workspace — keep it
isolated.

## Options evaluated

### Option A: Bevy 0.15 + Trunk + WebGL2 (chosen)

**Approach**: Standalone Bevy 0.15 binary in `apps/viewer/` with its own Cargo.toml
(not a workspace member). Built with Trunk. Renders via wgpu WebGL2 backend.

**Pros**:
- Bevy is the spec'd technology — validates the actual production path.
- Trunk is Bevy-recommended, zero-config WASM build + dev server.
- WebGL2 has universal browser support (Chrome, Firefox, Safari 15+, all iPads).
- Feature gating available: disable audio, gamepad, animation to cut binary size.
- glTF loading is well-tested in Bevy WASM community.

**Cons**:
- Binary size risk (may exceed 20 MB without optimization).
- Bevy's WASM ecosystem moves fast — could hit version-specific bugs.
- WebGL2 lacks some advanced features (compute shaders, storage buffers).

**Assessment**: Best fit. This is exactly what we need to validate. If it fails, we
know early.

### Option B: Bevy 0.15 + wasm-pack

**Approach**: Use wasm-pack instead of Trunk to compile and package.

**Pros**:
- More control over output structure (JS module, no HTML generation).
- Easier to embed as a JS module in SvelteKit directly (vs iframe).

**Cons**:
- More boilerplate: need manual index.html, JS init code, asset serving.
- Bevy examples and community all use Trunk — less documented path.
- wasm-pack's Bevy support is less battle-tested.
- For this spike, "just get it rendering" matters more than output format.

**Assessment**: Rejected for spike. Could revisit for T-013-02 if iframe approach
has issues, but Trunk is the path of least resistance.

### Option C: three.js / Babylon.js fallback

**Approach**: Skip Bevy, use a JS 3D library that loads glTF natively.

**Pros**:
- Tiny bundle size (~500 KB for three.js core).
- No WASM compilation. No binary size problem.
- Universal browser support, massive community.
- Could embed directly in SvelteKit (no iframe needed).

**Cons**:
- Abandons the Rust rendering pipeline from the spec.
- No code sharing between scene generator (Rust) and viewer.
- Lower rendering quality for PBR (though three.js is good enough).
- Fundamental architecture change — defeats the purpose of the spike.

**Assessment**: Documented as fallback plan if Bevy WASM proves unviable. This is
what we'd switch to if binary size > 20 MB and can't be reduced, or if iPad Safari
has showstopper bugs.

## Detailed design decisions

### 1. Bevy feature gating

Default Bevy includes audio, animation, gamepad, UI, state management, and more.
For a viewer that just loads glTF and renders, we need:

**Enabled features** (minimal set for 3D PBR rendering + glTF loading):
- `bevy_asset` — asset loading
- `bevy_core_pipeline` — core render pipeline
- `bevy_pbr` — PBR materials and lighting
- `bevy_render` — rendering backend
- `bevy_gltf` — glTF loading
- `bevy_winit` — window management (canvas on WASM)
- `bevy_log` — logging (console.log on WASM)
- `webgl2` — WebGL2 backend (required for broad Safari support)

**Disabled** (via `default-features = false`):
- `bevy_audio`, `bevy_animation`, `bevy_gilrs` (gamepad)
- `bevy_ui`, `bevy_text`, `bevy_sprite`, `bevy_scene`
- `bevy_state`, `bevy_dev_tools`
- `serialize`, `multi_threaded` (WASM is single-threaded)

This should cut binary size significantly — audio and animation alone are large.

### 2. Build configuration for small WASM

In `apps/viewer/Cargo.toml`:
```toml
[profile.release]
opt-level = "z"       # Optimize for size
lto = true            # Link-time optimization
codegen-units = 1     # Single codegen unit for better optimization
strip = true          # Strip debug symbols
```

Post-build: `wasm-opt -Oz` via Trunk's built-in wasm-opt integration.

In `Trunk.toml`:
```toml
[build]
release = true
dist = "dist"

[[hooks]]
stage = "post_build"
command = "wasm-opt"
command_arguments = ["-Oz", "-o", "{{wasm_out}}", "{{wasm_out}}"]
```

Actually, Trunk 0.21+ handles wasm-opt automatically when `--release` is used. Just
need to ensure `wasm-opt` is installed (via `binaryen` package).

### 3. Test model: DamagedHelmet.glb

- Khronos glTF 2.0 sample model.
- 3.7 MB .glb with embedded textures.
- Full PBR: base color, normal map, metallic-roughness, emissive, occlusion.
- Good complexity for a spike — not trivial, not huge.
- Download from Khronos GitHub and place in `apps/viewer/assets/models/`.
- Alternative: create a minimal test .glb if download is impractical.

### 4. App structure (minimal for spike)

```
apps/viewer/
  Cargo.toml          — Bevy dep with feature gating, release profile
  Trunk.toml          — Trunk build config
  index.html          — HTML shell (canvas, loading indicator)
  src/
    main.rs           — App entry, plugin setup, startup system
    camera.rs         — Orbit camera (drag-rotate, scroll-zoom)
    scene.rs          — glTF loading + spawn
    lighting.rs       — Directional light + ambient light
  assets/
    models/
      test_scene.glb  — DamagedHelmet or similar test model
```

This is the minimum viable spike. `interaction.rs` and `tiers.rs` from the spec are
T-013-02 scope.

### 5. Camera approach

Use `bevy_panorbit_camera` crate (lightweight orbit camera for Bevy):
- Drag to rotate, scroll to zoom, middle-click to pan.
- Touch support (pinch-zoom, drag-rotate) on mobile.
- ~2 KB additional WASM overhead — negligible.
- Alternative: write custom orbit camera. More control but more code for a spike.

Decision: use `bevy_panorbit_camera` for the spike. If it causes binary bloat or
compatibility issues, replace with custom implementation.

### 6. FPS measurement

Use Bevy's `FrameTimeDiagnosticsPlugin` + `LogDiagnosticsPlugin`:
- Logs FPS to console every 2 seconds.
- On WASM, this goes to browser console (via `bevy_log` → `console.log`).
- Sufficient for spike measurement. No need for an on-screen overlay.

### 7. What this spike does NOT do

- No postMessage bridge (T-013-02).
- No tap-to-inspect (T-013-02).
- No tier toggling (T-013-02).
- No sunlight slider (later ticket).
- No measurement tool (later ticket).
- No CI integration (follow-up).
- No deployment to Cloudflare Pages (follow-up).
- No integration with pt-scene output (pt-scene doesn't exist yet).

### 8. Verification criteria

| Metric | Target | Fail threshold |
|--------|--------|----------------|
| .wasm binary size (after wasm-opt) | < 15 MB | > 20 MB |
| Time to first frame (desktop Chrome) | < 5 sec | > 10 sec |
| Steady-state FPS (desktop Chrome) | > 30 FPS | < 15 FPS |
| Chrome rendering | Works | — |
| Firefox rendering | Works | — |
| Safari desktop rendering | Works | — |
| iPad Safari rendering | Works (or documented failure) | — |

### 9. Fallback plan

If Bevy WASM binary > 20 MB after all optimizations, or iPad Safari has showstopper
rendering bugs:

1. Document exact failure mode and measurements.
2. Prototype a three.js viewer as comparison (small spike — 1-2 hours).
3. Document three.js measurements for comparison.
4. Recommend path forward in review.md.
5. Update E-006 success criteria based on findings.

## Scenarios affected

- **S.2.4** (3D preview per tier): This spike proves the rendering layer. Combined with
  T-013-02 (iframe embedding), S.2.4 can reach ★★.
- **S.4.1** (3D viewer on tablet): iPad Safari testing in this spike directly informs
  feasibility.
- **S.4.3** (Material callouts): Requires tap-to-inspect which is T-013-02 scope, but
  depends on the viewer working at all.
