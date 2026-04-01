# Progress — T-002-01 pt-project & pt-materials

## Completed

### pt-materials
- [x] Cargo.toml: added serde_json as dev-dependency
- [x] types.rs: MaterialId, MaterialCategory, Unit, ExtrusionBehavior, Material
- [x] builder.rs: MaterialBuilder with method chaining and defaults
- [x] lib.rs: module declarations and re-exports
- [x] 11 tests passing

### pt-project
- [x] Cargo.toml: added pt-materials, pt-geo, rust_decimal dependencies
- [x] error.rs: ProjectError enum with Display and Error
- [x] serde_helpers.rs: geojson_polygon serde module for Polygon<f64>
- [x] types.rs: ZoneId, ZoneType, Zone, TierLevel, MaterialAssignment, AssignmentOverrides, Tier, ProjectStatus
- [x] project.rs: Project struct, zone CRUD, status transitions, tier access
- [x] geojson_conv.rs: to_geojson/from_geojson FeatureCollection conversion
- [x] lib.rs: module declarations and re-exports
- [x] 28 tests passing

### Verification
- [x] `cargo test --workspace` — 60 tests passing (21 pt-geo + 11 pt-materials + 28 pt-project)
- [x] `cargo clippy --workspace -- -D warnings` — clean
- [x] `cargo run -p pt-scenarios` — 0/240 min (no regressions, same as baseline)

## Deviations from Plan

1. **rust_decimal dependency on pt-project**: Added to Cargo.toml because
   AssignmentOverrides uses Decimal for price_override. Not in original stub.

2. **Default impls**: Clippy required Default for MaterialId, ZoneId, and Project
   (new_without_default lint). Added these.

3. **unwrap_or_default**: Clippy preferred this over `unwrap_or_else(MaterialId::new)`
   in the builder.

## Remaining

Nothing — all acceptance criteria covered. Ready for review.
