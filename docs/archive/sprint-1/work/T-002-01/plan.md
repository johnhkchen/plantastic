# Plan — T-002-01 pt-project & pt-materials

## Step 1: pt-materials types

**Files**: `crates/pt-materials/src/types.rs`, `crates/pt-materials/src/lib.rs`

Implement all type definitions:
- `MaterialId` newtype over Uuid with new(), derive macros
- `MaterialCategory` enum (Hardscape, Softscape, Edging, Fill)
- `Unit` enum (SqFt, CuYd, LinearFt, Each)
- `ExtrusionBehavior` tagged enum with associated data
- `Material` struct with all fields

Tests:
- MaterialId::new() uniqueness
- Serde round-trip for each enum (all variants)
- Full Material serde round-trip

**Verify**: `cargo test -p pt-materials`

## Step 2: pt-materials builder

**Files**: `crates/pt-materials/src/builder.rs`

Implement MaterialBuilder:
- `Material::builder(name, category)` constructor
- Method chain for each optional field
- `build()` returns Material with defaults for unspecified fields

Tests:
- Builder with required fields only → valid defaults
- Builder with all fields → all values correct
- Builder produces distinct IDs per build (default id)

**Verify**: `cargo test -p pt-materials`

## Step 3: Update pt-materials Cargo.toml

**Files**: `crates/pt-materials/Cargo.toml`

Add serde_json as dev-dependency for tests.

**Verify**: `cargo check -p pt-materials`

## Step 4: pt-project error types

**Files**: `crates/pt-project/src/error.rs`

Implement ProjectError enum with Display and Error impls.

**Verify**: `cargo check -p pt-project` (after step 5)

## Step 5: pt-project types

**Files**: `crates/pt-project/src/types.rs`

Implement:
- `ZoneId` newtype
- `ZoneType` enum
- `Zone` struct (with geometry as `Polygon<f64>`)
- `TierLevel` enum
- `MaterialAssignment` struct
- `AssignmentOverrides` struct
- `Tier` struct
- `ProjectStatus` enum

Defer Polygon serde to step 6 (serde_helpers). Zone will use
`#[serde(with = "crate::serde_helpers::geojson_polygon")]` on geometry field.

**Verify**: `cargo check -p pt-project`

## Step 6: Polygon serde helper

**Files**: `crates/pt-project/src/serde_helpers.rs`

Implement `geojson_polygon` module:
- serialize: Polygon<f64> → geojson::Geometry → serde
- deserialize: serde → geojson::Geometry → Polygon<f64>

Uses `geojson::Value::from(geo::Geometry::from(polygon))` and reverse.

Tests:
- Simple rectangle polygon serialize → deserialize round-trip
- Coordinate precision preserved

**Verify**: `cargo test -p pt-project`

## Step 7: Project struct and methods

**Files**: `crates/pt-project/src/project.rs`

Implement:
- `Project` struct with all fields
- `Project::new()` — empty draft with 3 default tiers
- `Project::with_id(id)` — same but with given ID
- Zone CRUD: add_zone, remove_zone, get_zone, get_zone_mut
- `transition_to()` with validation
- `tier()` and `tier_mut()` accessors

Tests:
- new() → Draft status, 3 tiers
- add_zone + get_zone
- add duplicate zone → error
- remove zone → ok, re-get → None
- remove nonexistent → error
- Valid transitions (Draft→Quoted, Quoted→Approved, Approved→Complete, Any→Draft)
- Invalid transitions (Draft→Complete, Draft→Approved)
- tier(Good) returns correct tier

**Verify**: `cargo test -p pt-project`

## Step 8: GeoJSON conversion

**Files**: `crates/pt-project/src/geojson.rs`

Implement to_geojson and from_geojson on Project:
- Zone → Feature (geometry + properties)
- Project → FeatureCollection (zones as features, metadata as foreign members)
- Round-trip: Project → GeoJSON → Project

Tests:
- Empty project round-trip
- Project with multiple zones → GeoJSON → Project
- Geometry coordinates preserved
- Zone properties (id, type, label) preserved
- Status and tiers preserved through foreign members

**Verify**: `cargo test -p pt-project`

## Step 9: Update pt-project Cargo.toml

**Files**: `crates/pt-project/Cargo.toml`

Add pt-materials and pt-geo as path dependencies.

**Verify**: `cargo check -p pt-project`

Note: This step is done early (before step 5) since types.rs needs MaterialId.
Reordering: do step 9 before step 4.

## Step 10: Update lib.rs files

**Files**: `crates/pt-project/src/lib.rs`, `crates/pt-materials/src/lib.rs`

Add module declarations and re-exports for all public types.

## Step 11: Full verification

Run the complete quality gate:
```
cargo test --workspace
cargo run -p pt-scenarios
just lint  (if available)
```

Confirm:
- All pt-materials tests pass
- All pt-project tests pass
- No regressions in pt-geo tests
- Scenario dashboard: still 0/240 min (no scenarios should regress)
- Clippy clean

## Execution Order (adjusted for dependencies)

1. pt-materials: Cargo.toml update (step 3)
2. pt-materials: types.rs + lib.rs (steps 1, 10a)
3. pt-materials: builder.rs (step 2)
4. Verify: `cargo test -p pt-materials`
5. pt-project: Cargo.toml update (step 9)
6. pt-project: error.rs (step 4)
7. pt-project: serde_helpers.rs (step 6)
8. pt-project: types.rs (step 5)
9. pt-project: project.rs (step 7)
10. pt-project: geojson.rs (step 8)
11. pt-project: lib.rs (step 10b)
12. Verify: `cargo test -p pt-project`
13. Full workspace verify (step 11)

## Risk Assessment

- **GeoJSON round-trip precision**: geo::Polygon<f64> → geojson → Polygon<f64> may
  lose precision at f64 boundaries. Mitigation: test with exact coordinates, accept
  f64 representation limits.
- **geojson crate API**: v0.24 API for constructing Features/FeatureCollections may
  have changed. Mitigation: check docs during implementation.
- **Serde for Polygon<f64>**: geo types don't derive Serde by default. Must use
  geojson crate as intermediate. If this is too complex, fall back to storing
  coordinates as `Vec<[f64; 2]>` with conversion helpers.
