# T-031-01 Review: pt-scene crate

## Summary

Created `crates/pt-scene/` — a pure-computation crate that generates glTF 2.0 binary (.glb) scenes from project zones and material assignments. Each zone polygon is extruded into a 3D mesh based on its material's `ExtrusionBehavior`, with solid PBR base colors per `MaterialCategory`. Output is ready for the Bevy viewer's tap-to-inspect protocol.

## Files created

| File | Lines | Purpose |
|------|-------|---------|
| `crates/pt-scene/Cargo.toml` | 23 | Crate manifest; deps: earcutr, pt-geo, pt-materials, pt-project |
| `crates/pt-scene/src/lib.rs` | 13 | Module root, public re-exports |
| `crates/pt-scene/src/error.rs` | 20 | `SceneError` enum: MissingMaterial, Triangulation, Export |
| `crates/pt-scene/src/mesh.rs` | ~180 | Zone polygon → extruded 3D triangle mesh (earcutr triangulation) |
| `crates/pt-scene/src/glb.rs` | ~230 | GLB binary assembly: multi-mesh, named nodes, PBR materials |
| `crates/pt-scene/src/scene.rs` | ~330 | Public API: `generate_scene()` orchestrator + 6 unit tests |

## Files modified

| File | Change |
|------|--------|
| `tests/scenarios/src/progress.rs` | Claimed milestone: pt-scene delivered by T-031-01, note written |

## Test coverage

**17 unit tests**, all passing in <1s:

| Module | Tests | What they verify |
|--------|-------|-----------------|
| mesh | 6 | Vertex/index counts for square + triangle, SitsOnTop/BuildsUp/Fills heights (independent arithmetic), empty polygon |
| glb | 5 | GLB magic+version, JSON chunk parseable with correct structure, multi-mesh distinct names, empty scene validity, material deduplication |
| scene | 6 | Single zone → valid GLB, multi-zone all present, empty → no crash, missing material → error, extrusion height matches spec (1.5in/12=0.125ft), label-less zone uses ID |

**Testing philosophy compliance:**
- Heights verified with independent arithmetic (not calling code under test)
- No mocks — uses real pt-geo, pt-materials, pt-project types
- Every `timed()` wrapper enforces 10s timeout
- No `#[ignore]` — all tests run in CI

## Acceptance criteria checklist

- [x] `crates/pt-scene/` crate created
- [x] `generate_scene(zones, assignments, materials, tier) -> Result<SceneOutput, SceneError>`
- [x] SceneOutput: `glb_bytes: Vec<u8>`, `metadata: SceneMetadata`
- [x] SitsOnTop: extrude upward by height_inches (converted to feet)
- [x] Fills: extrude downward (sunken bed)
- [x] BuildsUp: extrude upward
- [x] MaterialCategory → base color (Hardscape=gray, Softscape=brown, Edging=dark gray, Fill=tan)
- [x] Each zone mesh named by zone label (falls back to zone_id)
- [x] GLB output: valid glTF 2.0 binary (magic bytes 0x46546C67)
- [x] Unit tests: single zone, multiple zones, empty zones, extrusion height
- [x] `cargo clippy -p pt-scene -- -D warnings` passes
- [x] `cargo fmt -p pt-scene -- --check` passes

## Architecture decisions

1. **earcutr** for polygon triangulation — Mapbox earcut lineage, handles simple polygons
2. **Hand-rolled GLB** (like pt-scan) — serde_json for JSON chunk, manual binary buffer
3. **Solid colors only** — PBR textures deferred to follow-up (texture_ref field exists but unused)
4. **Y-up coordinate system** — poly X→scene X, poly Y→scene Z, height→scene Y
5. **Material deduplication** — same category color → shared glTF material

## Open concerns

1. **Pre-existing compile error**: `plantastic-api` has a missing match arm for `ProposalError::Render`. Not introduced by this ticket; exists in the working tree's unstaged changes from prior work.
2. **No hole support**: earcutr supports polygon holes, but `extrude_zone` currently only uses the exterior ring. Interior rings (e.g., a patio with a tree well) would need wall generation for inner boundaries. Low priority — no current zone types have holes.
3. **Texture_ref unused**: The material's `texture_ref` field is ignored; only solid base colors are applied. This is per spec ("solid colors per category is fine for ★☆☆☆☆").

## Scenario impact

- **Milestone claimed**: "pt-scene: 3D scene generation from project model" → unlocks S.2.4, S.4.1
- **S.2.4 (3D preview)**: path to ThreeStar now requires wiring `generate_scene` into an API route (T-031-02)
- **S.4.1 (crew viewer)**: scene generation available for crew-facing glTF export
- No scenario regressions — pt-scene is additive (new crate, no existing code modified)
