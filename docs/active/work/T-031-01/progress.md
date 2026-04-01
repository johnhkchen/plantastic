# T-031-01 Progress: pt-scene crate

## Completed

### Step 1: Scaffold crate ✓
- Created `crates/pt-scene/Cargo.toml` with workspace deps + earcutr
- Created `src/lib.rs`, `src/error.rs` with module structure
- `cargo check -p pt-scene` passes

### Step 2: Implement mesh.rs ✓
- `extrude_zone()` with top face, bottom face, side walls
- Coordinate mapping: poly X→scene X, poly Y→scene Z, height→scene Y
- earcutr polygon triangulation for faces
- `compute_normals()` smooth per-vertex normals
- 6 unit tests pass (vertex counts, index counts, height specs, fills, empty)

### Step 3: Implement glb.rs ✓
- `to_glb()` assembles multi-mesh GLB with named nodes and PBR materials
- Material deduplication by base color
- `category_base_color()` maps MaterialCategory → solid color
- Empty scene → valid GLB with no meshes
- 5 unit tests pass (magic bytes, JSON parsing, multi-mesh names, empty, dedup)

### Step 4: Implement scene.rs ✓
- `generate_scene(zones, assignments, materials, tier)` orchestrator
- Filters assignments, looks up materials, extrudes zones, assembles GLB
- Zone naming: label if present, else zone_id string
- 6 unit tests pass (single zone, multi zone, empty, missing material, height spec, label fallback)

### Step 5: Integration and quality gate ✓
- `cargo fmt -p pt-scene` — clean
- `cargo clippy -p pt-scene -- -D warnings` — clean
- `cargo test -p pt-scene` — 17/17 pass
- Milestone updated in `tests/scenarios/src/progress.rs`

## Deviations from plan

None. Implementation followed the plan exactly.

## Notes

- Pre-existing compile error in `plantastic-api` (missing match arm for `ProposalError::Render`) — not introduced by this ticket, not in scope to fix.
