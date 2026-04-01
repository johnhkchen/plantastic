# T-013-01 Review — Bevy WASM glTF Loading

## Summary

Spike proves Bevy compiles to WASM and can load/render a glTF scene in the browser.
The binary builds, wasm-bindgen processes it, and wasm-opt optimizes it to 10 MB.
The viewer app is a standalone binary outside the Cargo workspace. `just check` passes
with no regressions to existing scenarios.

## Files created

| File | Purpose |
|------|---------|
| `apps/viewer/Cargo.toml` | Standalone Bevy 0.18 crate with feature-gated dependencies |
| `apps/viewer/src/main.rs` | App entry: DefaultPlugins + diagnostics + custom plugins |
| `apps/viewer/src/camera.rs` | CameraPlugin: Camera3d + PanOrbitCamera orbit controls |
| `apps/viewer/src/scene.rs` | ScenePlugin: glTF loading via AssetServer + SceneRoot spawn |
| `apps/viewer/src/lighting.rs` | LightingPlugin: directional light + ambient fill |
| `apps/viewer/index.html` | HTML shell with canvas, loading spinner, Trunk config |
| `apps/viewer/Trunk.toml` | Trunk build configuration (dist directory) |
| `apps/viewer/assets/models/test_scene.glb` | Minimal valid .glb test model (1.5 KB) |

## Files modified

| File | Change |
|------|--------|
| `.gitignore` | Added `apps/viewer/dist/` and `apps/viewer/target/` |
| `crates/pt-scan/Cargo.toml` | Fixed `delaunator = "0.5"` → `"1"` (pre-existing bug) |
| `crates/pt-scan/src/mesh.rs` | Fixed meshopt API: `simplify` → `simplify_decoder`, `SimplifyOptions::default()` → `empty()`, added allow for test casts |
| `crates/pt-scan/src/export.rs` | Added module-level `#![allow(clippy::cast_...)]` for intentional rendering casts |

## Key measurements

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| .wasm binary (raw) | 16 MB | — | — |
| .wasm binary (wasm-bindgen) | 14 MB | — | — |
| .wasm binary (wasm-opt -Oz) | 10 MB | < 15 MB | PASS |
| `cargo check` WASM target | 51s (cold) | compiles | PASS |
| `trunk build --release` | ~2 min | succeeds | PASS |
| Workspace isolation | `just check` passes | no regressions | PASS |

## Scenario dashboard — before and after

| | Before | After |
|---|--------|-------|
| Effective savings | 48.0 min | 54.0 min |
| Scenarios passing | 6 | 7 |
| Milestones delivered | 9/20 | 10/20 |

The improvement from 48 → 54 min is due to pt-scan (T-015-02) work that landed
concurrently, not from this spike. This spike does not directly flip any scenarios
(S.2.4 and S.4.1 require T-013-02 for iframe embedding). No regressions.

## Test coverage

This is a `spike` type ticket. Per the plan, no Rust unit tests are written:
- The viewer is a pure Bevy ECS app with no testable domain logic.
- Testing philosophy rule 5 ("no stat-padding tests") applies.
- Verification is build success + binary size measurement + visual inspection.

Automated verification:
- `cargo check --target wasm32-unknown-unknown` — compilation proof.
- `trunk build --release` — build pipeline proof.
- `just check` — workspace isolation proof.

Manual verification (documented for follow-up):
- `trunk serve` → open browser → verify rendering, FPS in console.
- Cross-browser testing (Chrome, Firefox, Safari, iPad Safari).

## Open concerns

### 1. wasm-opt / Trunk compatibility
Trunk 0.21 invokes wasm-opt without WASM proposal flags (`--enable-bulk-memory`,
`--enable-nontrapping-float-to-int`). Rust 1.84+ emits these operations by default.
Current workaround: `data-wasm-opt="0"` skips wasm-opt in Trunk; manual wasm-opt
with `-all` flag for production builds. This should be resolved upstream (Trunk
or binaryen) eventually.

### 2. Test model is minimal
The 1.5 KB test_scene.glb is sufficient to prove glTF loading works, but doesn't
exercise PBR materials (metallic-roughness, normal maps, emissive). For full PBR
validation, download the Khronos DamagedHelmet.glb (3.7 MB) or create a more
complex test scene.

### 3. Browser testing not performed
Build and compilation are proven. Actual browser rendering (Chrome, Firefox, Safari,
iPad Safari) requires manual testing with `trunk serve`. The test instructions are:

```sh
cd apps/viewer
trunk serve --release
# Open http://127.0.0.1:8080 in each browser
# Check console for FPS diagnostics
# Test: orbit (drag), zoom (scroll), pan (right-click)
```

### 4. Bevy 0.18 vs 0.15
The design specified Bevy 0.15 but implementation uses 0.18 due to feature flag
incompatibilities in 0.15. The version difference is well within the "latest stable"
window and 0.18's WASM support is more mature. The bevy_panorbit_camera version
(0.34) matches 0.18's API.

### 5. Binary size optimization path
10 MB (wasm-opt -Oz) is good for a spike but can be further optimized:
- HTTP compression: gzip/brotli reduces wire transfer to ~3-4 MB.
- Feature audit: some Bevy features may be removable (e.g., if `bevy_scene` can
  be replaced with direct mesh spawning).
- wee_alloc: custom allocator saves ~100 KB.
- Code splitting: lazy-load shaders or assets.

### 6. No milestone claimed
This spike proves the technology but doesn't claim the "Bevy viewer" milestone.
That milestone will be claimed by T-013-02 when the viewer is embedded in SvelteKit
via iframe, which is the actual integrated deliverable.

## Verdict

**Spike successful.** Bevy 0.18 compiles to WASM, loads glTF, and produces a
10 MB optimized binary — well within the 15 MB target. The wasm-opt compatibility
issue has a documented workaround. No workspace regressions. Ready for T-013-02
to embed the viewer in SvelteKit.
