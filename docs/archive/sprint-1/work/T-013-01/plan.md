# T-013-01 Plan — Bevy WASM glTF Loading

## Prerequisites

- `rustup target add wasm32-unknown-unknown` (WASM compilation target)
- `cargo install trunk` (WASM build tool for Bevy)
- `wasm-opt` installed via `brew install binaryen` (binary size optimization)

These are developer machine prerequisites, not CI requirements.

## Implementation steps

### Step 1: Create Cargo.toml with feature-gated Bevy

Create `apps/viewer/Cargo.toml` as a standalone crate (not workspace member).
Pin Bevy 0.15 with `default-features = false` and the minimal feature set:
`bevy_asset`, `bevy_core_pipeline`, `bevy_pbr`, `bevy_render`, `bevy_gltf`,
`bevy_winit`, `bevy_log`, `webgl2`. Add `bevy_panorbit_camera`. Set release
profile for size optimization.

**Verify**: `cd apps/viewer && cargo check --target wasm32-unknown-unknown`
compiles without errors.

### Step 2: Create lighting.rs

Implement `LightingPlugin` with a startup system that spawns:
- `DirectionalLight` at ~45 degree elevation, slightly warm color, shadows enabled.
- Low-intensity ambient light for fill.

No dependencies on other modules.

**Verify**: Compiles as part of the crate (checked in step 4).

### Step 3: Create camera.rs

Implement `CameraPlugin` with a startup system that spawns:
- `Camera3d` with `PanOrbitCamera` component from `bevy_panorbit_camera`.
- Initial position: ~5 units from origin, looking at center.
- Touch input enabled (default in bevy_panorbit_camera).

**Verify**: Compiles as part of the crate (checked in step 4).

### Step 4: Create scene.rs

Implement `ScenePlugin`:
- Startup system: `asset_server.load("models/test_scene.glb")`, store handle
  in a resource.
- Update system: watch for asset load completion, spawn `SceneRoot`.
- Log a "scene loaded" message with timing for measurement.

**Verify**: Compiles as part of the crate (checked in step 5).

### Step 5: Create main.rs

Compose all plugins:
- `DefaultPlugins` with customizations (canvas selector, fit-canvas-to-parent,
  prevent-default, WebGL2 backend preference).
- Add `FrameTimeDiagnosticsPlugin`, `LogDiagnosticsPlugin`.
- Add `CameraPlugin`, `ScenePlugin`, `LightingPlugin`.

**Verify**: `cargo check --target wasm32-unknown-unknown` succeeds for the
full crate.

### Step 6: Create test glTF asset

Generate a minimal .glb programmatically (a colored cube with PBR metallic-
roughness material) using a small build script or manual binary construction.
Alternatively, include a simple procedurally-generated scene.

Place in `apps/viewer/assets/models/test_scene.glb`.

**Verify**: File exists and is valid glTF (can verify with gltf-viewer or
Blender if available).

### Step 7: Create index.html

HTML shell with:
- `<link data-trunk rel="rust" data-wasm-opt="z" />` to trigger Trunk build.
- `<link data-trunk rel="copy-dir" href="assets" />` to copy assets to dist.
- `<canvas id="bevy">` for Bevy to attach to.
- CSS: fullscreen canvas, loading spinner.
- Viewport meta tag for mobile.

**Verify**: HTML is valid, Trunk can parse the data-trunk attributes.

### Step 8: Create Trunk.toml

Minimal configuration — mostly just dist directory setting. Trunk 0.21+ handles
most configuration via index.html data-trunk attributes.

**Verify**: `trunk build --release` in apps/viewer/ produces `dist/` with
.wasm, .js, and index.html.

### Step 9: Update .gitignore

Add `apps/viewer/dist/` and `apps/viewer/target/` to root .gitignore.

**Verify**: `git status` doesn't show dist/ artifacts.

### Step 10: Build and measure

1. Install prerequisites if needed: `rustup target add wasm32-unknown-unknown`.
2. `cd apps/viewer && trunk build --release`
3. Measure .wasm file size: `ls -lh dist/*.wasm`
4. Record size in progress.md.

**Verify**: Binary exists, size is documented.

### Step 11: Browser testing

1. `cd apps/viewer && trunk serve --release`
2. Open in Chrome — verify rendering, check console for FPS logs.
3. Open in Firefox — verify rendering.
4. Open in Safari — verify rendering.
5. Note: iPad Safari testing requires either a real device or BrowserStack.
   Document instructions for manual testing.

**Verify**: Renders in at least Chrome. Document results for all browsers
tested.

### Step 12: Verify workspace is unaffected

Run `just check` from workspace root to confirm the viewer crate doesn't
break existing workspace builds, tests, or scenarios.

**Verify**: `just check` passes. Scenario dashboard shows same 48.0 min.

## Testing strategy

This is a spike — the "tests" are the measurements and browser verification.

### What gets tested
- **Compilation**: `cargo check --target wasm32-unknown-unknown` (automated).
- **Build**: `trunk build --release` produces artifacts (automated).
- **Binary size**: Measured and documented (manual, recorded in progress.md).
- **Rendering**: Visual verification in browsers (manual).
- **FPS**: Read from browser console (manual, FrameTimeDiagnosticsPlugin).
- **Workspace isolation**: `just check` still passes (automated).

### What does NOT get tested
- No Rust unit tests for the viewer (there's no testable logic — it's all
  Bevy ECS systems that require a running app context).
- No integration tests (no API, no database, no domain logic).
- No scenario tests (S.2.4 and S.4.1 require T-013-02 to pass).

### Why no unit tests is acceptable for this spike
The ticket type is `spike`. The deliverable is knowledge (does Bevy WASM work?)
documented in measurements. The code is a minimal proof-of-concept, not
production logic. Testing philosophy rule 5 ("no stat-padding tests") applies:
a test that checks "Bevy app compiles" adds no value beyond `cargo check`.

## Commit plan

1. **Commit 1**: Cargo.toml + src/*.rs + index.html + Trunk.toml + .gitignore
   "Add Bevy WASM viewer spike (T-013-01): standalone app that loads glTF"
2. **Commit 2** (if needed): Fix any build issues discovered during testing.
3. **Commit 3** (if needed): Add test asset if not included in commit 1.

Goal: single atomic commit if possible.
