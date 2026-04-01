# T-031-01 Plan: pt-scene crate

## Step 1: Scaffold crate

- Create `crates/pt-scene/Cargo.toml` with workspace deps
- Create `crates/pt-scene/src/lib.rs` with module declarations
- Create `crates/pt-scene/src/error.rs` with SceneError enum
- Verify: `cargo check -p pt-scene`

## Step 2: Implement mesh.rs — polygon extrusion

- `triangulate_polygon()`: convert geo::Polygon exterior ring to earcutr format, triangulate, return positions + indices
- `extrude_walls()`: for each edge of the exterior ring, generate 2 triangles connecting top and bottom
- `extrude_zone()`: combine top face, bottom face, walls; apply Y-up coordinate transform (poly_x→X, poly_y→Z, height→Y)
- `compute_normals()`: smooth per-vertex normals (port from pt-scan pattern)
- Unit tests:
  - Square polygon → correct vertex count (walls + caps)
  - Triangle polygon → correct index count
  - SitsOnTop extrusion → top face at expected Y
  - BuildsUp extrusion → same behavior as SitsOnTop
  - Fills extrusion → geometry below Y=0

## Step 3: Implement glb.rs — GLB binary assembly

- `category_base_color()`: MaterialCategory → [f32; 4] RGBA
- `to_glb()`: build JSON + binary buffer for multiple named meshes with materials
  - Deduplicate materials by base_color
  - One bufferView per mesh attribute set (positions, normals, indices)
  - One node per mesh, named
  - Pad JSON to 4-byte alignment, pad binary to 4-byte alignment
  - Assemble header + JSON chunk + binary chunk
- Handle empty meshes list → valid empty glTF
- Unit tests:
  - GLB magic bytes and version correct
  - JSON chunk parseable, contains expected fields
  - Multiple meshes → multiple nodes with distinct names
  - Empty meshes → valid GLB header

## Step 4: Implement scene.rs — orchestrator

- `generate_scene()`: filter assignments, lookup materials, extrude zones, assemble GLB
- Handle edge cases: empty zones, missing materials, zones without assignments
- Unit tests:
  - Single zone with assignment → valid GLB, metadata.zone_count=1
  - Multiple zones → all meshes present, correct triangle_count
  - Empty zones → empty scene, no error
  - Missing material → SceneError::MissingMaterial
  - Extrusion height matches material spec (independently computed)

## Step 5: Integration and quality gate

- Run `just check` (fmt + lint + test + scenarios)
- Fix any clippy warnings
- Update milestone in `tests/scenarios/src/progress.rs`
- Verify scenario dashboard output

## Testing strategy

| Test | Type | What it verifies |
|------|------|-----------------|
| square extrusion vertex count | unit | mesh geometry correctness |
| triangle extrusion index count | unit | triangulation + walls |
| sits_on_top height | unit | extrusion Y position (independent arithmetic) |
| fills below grade | unit | negative Y extrusion |
| glb magic bytes | unit | valid glTF 2.0 binary format |
| glb json parseable | unit | JSON chunk structure |
| multi-mesh distinct names | unit | zone naming for viewer picking |
| empty scene | unit | no crash on empty input |
| single zone scene | unit | end-to-end: zone → GLB bytes |
| multi zone scene | unit | all zones present in output |
| missing material error | unit | error handling |
| extrusion height match | unit | height_inches/12 = feet (independent calc) |
