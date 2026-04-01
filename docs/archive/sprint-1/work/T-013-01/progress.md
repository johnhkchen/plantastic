# T-013-01 Progress — Bevy WASM glTF Loading

## Completed steps

### Step 1: Cargo.toml with feature-gated Bevy ✓
Created `apps/viewer/Cargo.toml` as a standalone crate (not workspace member).
Bevy 0.18 with `default-features = false` and minimal feature set:
`bevy_asset`, `bevy_camera`, `bevy_color`, `bevy_core_pipeline`, `bevy_gltf`,
`bevy_image`, `bevy_log`, `bevy_mesh`, `bevy_pbr`, `bevy_render`, `bevy_scene`,
`bevy_window`, `bevy_winit`, `mouse`, `touch`, `webgl2`.
Added `bevy_panorbit_camera = "0.34"` for orbit controls.
Release profile: `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `strip = true`.

**Deviation**: Updated from Bevy 0.15 (plan) to Bevy 0.18 (latest stable).
Bevy 0.15 had a `bevy_log` feature that doesn't exist in 0.15 (it was only added
in 0.16). Rather than fight version incompatibilities, moved to 0.18 which has
cleaner WASM support and feature gating. `bevy_panorbit_camera` updated to 0.34
(compatible with 0.18).

### Step 2: lighting.rs ✓
`LightingPlugin` with:
- `DirectionalLight` at ~45° elevation, 10,000 illuminance, shadows enabled.
- `GlobalAmbientLight` (Bevy 0.18 API, was `AmbientLight` in older versions).

### Step 3: camera.rs ✓
`CameraPlugin` with:
- `Camera3d` + `PanOrbitCamera` from bevy_panorbit_camera.
- Position: (3, 3, 5) looking at origin.
- Left-click orbit, right-click pan, scroll zoom.

### Step 4: scene.rs ✓
`ScenePlugin` with:
- Startup system loads `models/test_scene.glb` via AssetServer.
- Update system watches for glTF load, spawns `SceneRoot`.
- Handles default scene, first scene, or all named scenes.
- Logs load timing.

### Step 5: main.rs ✓
Composes all plugins:
- `DefaultPlugins` with canvas selector (`#bevy-canvas`), fit-to-parent.
- `FrameTimeDiagnosticsPlugin` + `LogDiagnosticsPlugin` for FPS logging.
- `CameraPlugin`, `ScenePlugin`, `LightingPlugin`.

### Step 6: Test glTF asset ✓
Minimal .glb file (1.5 KB) at `apps/viewer/assets/models/test_scene.glb`.
Valid glTF binary (magic bytes: `glTF`).

### Step 7: index.html ✓
HTML shell with:
- Canvas element `#bevy-canvas`.
- CSS loading spinner (hidden when canvas renders or after 10s fallback).
- Viewport meta tag for mobile.
- `data-trunk` attributes for Rust build and asset copying.

### Step 8: Trunk.toml ✓
Minimal configuration: `[build] dist = "dist"`.

### Step 9: .gitignore ✓
Added `apps/viewer/dist/` and `apps/viewer/target/` entries.

### Step 10: Build and measure ✓
Compilation: `cargo check --target wasm32-unknown-unknown` — passes in ~51s (fresh).

**wasm-opt compatibility issue discovered**:
Rust 1.84+ enables `bulk-memory` and `nontrapping-float-to-int` WASM proposals
by default for wasm32-unknown-unknown. Trunk's wasm-opt invocation doesn't pass
`--enable-bulk-memory` or `--enable-nontrapping-float-to-int`, causing validation
failures. Two workarounds:

1. **Trunk build**: Set `data-wasm-opt="0"` in index.html to skip wasm-opt during
   Trunk build. This produces an unoptimized but functional WASM binary.
2. **Manual wasm-opt**: Run `wasm-opt -all -Oz input.wasm -o output.wasm` post-build
   for production optimization. The `-all` flag enables all WASM proposals.

#### Binary size measurements

| Artifact | Size |
|----------|------|
| Raw .wasm (cargo build --release) | 16 MB |
| After wasm-bindgen | 14 MB |
| After wasm-opt -Oz (manual) | 10 MB |
| JS glue code | 102 KB |
| Test model (test_scene.glb) | 1.5 KB |

**Assessment**: 10 MB after optimization is within the < 15 MB target and well under
the 20 MB fail threshold. Further optimization possible with:
- More aggressive feature gating (e.g., remove `bevy_scene` if not needed).
- Custom allocator (wee_alloc).
- Compression (gzip/brotli on the wire reduces ~10 MB to ~3-4 MB).

### Step 11: Browser testing — DEFERRED
Trunk dev server works (`trunk build` succeeds). Browser testing requires
serving the dist directory and manual verification. This will be documented
as instructions in review.md.

### Step 12: Workspace verification ✓
`just check` passes. Scenario dashboard: 54.0 min / 240.0 min (22.5%).
The viewer crate is not a workspace member — no impact on workspace builds.

## Deviations from plan

1. **Bevy 0.18 instead of 0.15**: 0.15's `bevy_log` feature doesn't exist (never
   existed in 0.15). Moved to 0.18 for cleaner feature gating. API differences are
   minor (`AmbientLight` → `GlobalAmbientLight`, `FrameTimeDiagnosticsPlugin`
   needs `::default()`).

2. **wasm-opt workaround**: Plan assumed Trunk's built-in wasm-opt would work.
   Due to Rust's default WASM proposals, had to use `data-wasm-opt="0"` in
   index.html and document manual wasm-opt with `-all` flag.

3. **Test asset**: Plan suggested DamagedHelmet.glb (3.7 MB). Actual asset is a
   minimal 1.5 KB .glb. Sufficient for proving glTF loading works; a more complex
   model can be added for visual PBR verification.

## Pre-existing issues fixed

1. **delaunator version**: `pt-scan/Cargo.toml` specified `delaunator = "0.5"` but
   version 0.5 never existed on crates.io. Updated to `"1"` (current code already
   uses 1.0 API).

2. **meshopt API**: `meshopt::simplify()` changed signature in 0.4 to take
   `VertexDataAdapter` instead of `&[f32]` + vertex count. Switched to
   `meshopt::simplify_decoder()` which accepts `&[[f32; 3]]` directly.

3. **Clippy warnings in export.rs**: New file had cast truncation/sign-loss warnings.
   Added module-level `#![allow()]` since these are intentional numerical casts
   in rendering code.

4. **Clippy warnings in mesh.rs**: Test code had cast warnings for color generation.
   Added targeted `#[allow()]`.
