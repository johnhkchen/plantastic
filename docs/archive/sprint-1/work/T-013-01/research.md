# T-013-01 Research — Bevy WASM glTF Loading

## Ticket scope

Spike to prove Bevy compiles to WASM and can load/render a glTF scene in the browser.
Measure binary size, time to first frame, steady-state FPS. Test across Chrome, Firefox,
Safari desktop, and iPad Safari. If binary > 20 MB or iPad Safari fails, document the
failure mode and propose alternatives.

## What exists in the codebase

### Workspace structure
- Cargo workspace at root with `crates/*` and `tests/scenarios` as members.
- `apps/viewer/` exists but contains only `.gitkeep` — no code, no Cargo.toml.
- `apps/viewer/` is NOT a workspace member (workspace only includes `crates/*`).
- No Bevy dependency anywhere in the workspace.
- No WASM build tooling configured (no trunk, no wasm-pack, no wasm-bindgen).

### .cargo/config.toml
- Has build rustflags, test time limits, and aliases for `cargo t` / `cargo c`.
- No WASM target configuration.

### Existing rendering/3D references
- `docs/specification.md` Component 9 defines the viewer: Bevy → WASM, embedded via iframe.
  Orbit camera, tap-to-inspect, tier toggle, sunlight slider, measurement tool.
- Planned file structure in spec:
  - `apps/viewer/src/main.rs` — Bevy app entry
  - `apps/viewer/src/camera.rs` — orbit camera controls
  - `apps/viewer/src/scene.rs` — glTF scene loading
  - `apps/viewer/src/interaction.rs` — tap-to-inspect, measurement
  - `apps/viewer/src/tiers.rs` — tier toggle
  - `apps/viewer/src/lighting.rs` — sunlight direction control

### Frontend viewer placeholder
- `web/src/routes/(app)/project/[id]/viewer/+page.svelte` — "Coming soon" placeholder.
- T-013-02 will embed the viewer via iframe with postMessage bridge.

### Assets directory
- `assets/textures/` and `assets/models/` exist but are empty (placeholder `.gitkeep`).

### Related milestones
- "Bevy viewer: glTF loading + orbit + tap-to-inspect" — `delivered_by: None`, unlocks S.4.1 + S.4.3.
- "pt-scene: 3D scene generation from project model" — separate milestone, unlocks S.2.4 + S.4.1.

### Scenario dashboard baseline
- 48.0 effective min / 240.0 min (20.0%)
- S.2.4 (3D preview per tier) — NotImplemented, prereqs: 0/1 met
- S.4.1 (3D viewer on tablet) — NotImplemented, prereqs: 0/2 met

## Bevy WASM: current state of the ecosystem (as of early 2026)

### Bevy version landscape
- Bevy 0.15 is the latest stable release (Dec 2025).
- WASM support has been first-class since Bevy 0.10. By 0.15, the WASM story is mature:
  - `bevy_winit` handles canvas sizing and event loops on web.
  - `bevy_render` uses wgpu which maps to WebGPU (Chrome 113+) or falls back to WebGL2.
  - glTF loading via `bevy_gltf` works identically on native and WASM.

### Rendering backend: WebGPU vs WebGL2
- **WebGPU**: Chrome 113+ (stable since May 2023), Firefox Nightly (behind flag),
  Safari 18+ (macOS Sequoia / iOS 18). iPad Safari 18 supports WebGPU.
- **WebGL2**: Universal support (Chrome, Firefox, Safari 15+, all iPads).
- Bevy 0.15 defaults to WebGPU on web, with WebGL2 fallback via `WGPU_BACKEND=gl`.
- For this spike: target WebGL2 initially for maximum compatibility. WebGPU can be
  enabled later when Safari/Firefox support stabilizes.

### Binary size concerns
- A minimal Bevy 0.15 WASM build (3D PBR + glTF) produces ~20-30 MB unoptimized.
- With aggressive optimization the target is 8-15 MB:
  - `wasm-opt -Oz` (Binaryen) — typically 30-50% reduction.
  - Bevy feature gating — disable unused features (audio, animation, gamepad, UI, etc.).
  - `opt-level = "z"` + `lto = true` + `codegen-units = 1` in release profile.
  - `wasm-bindgen` strip + `strip = true` in Cargo.toml.
- E-006 success criteria: < 15 MB, ideally < 10 MB.

### Build tooling: trunk vs wasm-pack
- **trunk** (v0.21+): Bevy-recommended. Watches, builds, serves. Generates index.html
  with WASM + JS glue. Handles asset copying. One command: `trunk serve`.
  - Pros: zero-config for Bevy, auto-reload, asset pipeline.
  - Cons: another binary to install, less control over output structure.
- **wasm-pack**: More general-purpose. Produces npm-ready packages.
  - Pros: familiar to JS ecosystem, fine-grained control.
  - Cons: more manual setup for Bevy (need custom index.html, asset serving).
- **trunk is the clear choice** for this spike — it's what the Bevy community uses
  and has the least friction for a standalone viewer app.

### iPad Safari constraints
- iPad Safari 18+ supports WebGPU. Older iPads on Safari 15-17 need WebGL2.
- Touch events: Bevy's `bevy_winit` handles touch → pointer events on WASM.
- Performance: iPad Pro (M-series) handles PBR rendering well. Older iPads (A12/A13)
  may struggle with complex scenes but should handle a single test model fine.
- Canvas sizing: must handle devicePixelRatio for Retina displays.
- Audio context: Safari requires user interaction before audio — not relevant for viewer.

### glTF loading in Bevy
- `bevy_gltf` loads `.gltf` and `.glb` (binary glTF) files.
- On WASM, assets are fetched via HTTP (not filesystem). Trunk serves from an `assets/`
  directory that gets copied to the dist output.
- PBR materials (metallic-roughness workflow) work out of the box.
- Embedded textures in `.glb` files avoid extra HTTP requests — preferred for web.

## Constraints and assumptions

1. **apps/viewer/ is NOT in workspace members** — the spec shows it under `apps/`, not
   `crates/`. This is intentional: the viewer is a standalone binary targeting WASM,
   not a library crate consumed by the rest of the workspace. It won't share workspace
   lints or dependencies (Bevy's dependency tree is separate from Axum/sqlx).

2. **Test glTF model** — need a free, small (~1-5 MB) glTF model with PBR materials.
   Options: Khronos glTF sample models (DamagedHelmet, FlightHelmet), Sketchfab CC0
   models. DamagedHelmet.glb is 3.7 MB with full PBR — good spike candidate.

3. **No interaction complexity yet** — T-013-01 is just "load and render." Orbit camera
   is useful for visual verification but tap-to-inspect is T-013-02 scope.

4. **Measurement methodology** — binary size is the .wasm file after wasm-opt. Time to
   first frame measured via browser performance API or console.time. FPS via Bevy's
   built-in diagnostics (FrameTimeDiagnosticsPlugin).

5. **This spike does not need to integrate with the workspace build** — it's a standalone
   proof that the technology works. Integration (workspace member, CI, deployment) is
   follow-up work.

## Risks identified

| Risk | Severity | Mitigation |
|------|----------|------------|
| Binary size > 20 MB after optimization | High | Feature gating, wasm-opt, measure per-feature cost |
| iPad Safari WebGL2 rendering bugs | High | Test early, have three.js fallback plan documented |
| Trunk version incompatibility with Bevy 0.15 | Medium | Pin trunk version, use known-good combo |
| glTF asset loading fails on WASM | Low | Well-tested path in Bevy community |
| Touch events not working on iPad | Medium | Test pinch-zoom, drag — may need custom input handling |
| wgpu shader compilation slow on first load | Medium | Measure, consider shader pre-compilation |

## Key files to read/reference

- `docs/specification.md` lines 159-190 (Component 8 & 9)
- `docs/active/epics/E-006-3d-viewer-spike.md`
- `docs/active/stories/S-013-bevy-wasm-spike.md`
- `tests/scenarios/src/progress.rs` line 175 (Bevy viewer milestone)
- `tests/scenarios/src/suites/` (S.2.4, S.4.1 scenario definitions)
