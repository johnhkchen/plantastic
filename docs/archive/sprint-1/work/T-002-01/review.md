# Review — T-002-01 pt-project & pt-materials

## Summary

Implemented the two core domain crates that the entire Plantastic system operates on.
pt-materials defines what landscapers sell (materials, pricing, extrusion behavior).
pt-project defines the design model (projects, zones, tiers, material assignments)
with GeoJSON serialization for PostGIS/frontend interchange.

## Files Changed

### pt-materials (4 files)

| File | Action | Description |
|------|--------|-------------|
| `Cargo.toml` | Modified | Added serde_json dev-dependency |
| `src/lib.rs` | Modified | Module declarations and re-exports |
| `src/types.rs` | Created | MaterialId, Material, MaterialCategory, Unit, ExtrusionBehavior |
| `src/builder.rs` | Created | MaterialBuilder with method chaining and defaults |

### pt-project (7 files)

| File | Action | Description |
|------|--------|-------------|
| `Cargo.toml` | Modified | Added pt-materials, pt-geo, rust_decimal deps |
| `src/lib.rs` | Modified | Module declarations and re-exports |
| `src/error.rs` | Created | ProjectError enum |
| `src/serde_helpers.rs` | Created | Polygon<f64> ↔ GeoJSON serde module |
| `src/types.rs` | Created | ZoneId, Zone, ZoneType, Tier, TierLevel, MaterialAssignment, etc. |
| `src/project.rs` | Created | Project struct with zone CRUD, status transitions, tier access |
| `src/geojson_conv.rs` | Created | Project ↔ GeoJSON FeatureCollection conversion |

## Test Coverage

### pt-materials — 11 tests
- MaterialId: uniqueness, display
- MaterialCategory: serde round-trip (all 4 variants)
- Unit: serde round-trip (all 4 variants)
- ExtrusionBehavior: serde round-trip (SitsOnTop, Fills, BuildsUp)
- Material: full serde round-trip with all fields
- Builder: required-only defaults, all-fields, unique default IDs

### pt-project — 28 tests
- Zone CRUD: add, get, get_mut, remove, duplicate error, not-found error (6)
- Status transitions: valid forward, reset to draft, invalid transitions (4)
- Project construction: 3 tiers, Draft status (1)
- Tier access: correct tier returned, assignment push (1)
- Serde round-trip: Zone, ZoneType, TierLevel, ProjectStatus, full Project (6)
- Polygon serde helper: round-trip, coordinate precision (2)
- GeoJSON conversion: empty project, multi-zone, geometry preservation, property
  preservation, status/tiers preservation, invalid input error (6)
- Type-level: ZoneId uniqueness, status can_transition_to validation (2)

### Gap analysis
- No tests for AssignmentOverrides serde (minor — it's a simple struct with derive).
  If it breaks, the project JSON round-trip test would catch it.
- No integration tests crossing pt-geo → pt-project (e.g., computing area from
  a Zone's geometry). This is tested in scenarios via pt-quote (T-002-02).

## Scenario Dashboard

| Metric | Before | After |
|--------|--------|-------|
| Verified savings | 0.0 / 240.0 min | 0.0 / 240.0 min |
| Scenarios passing | 0 | 0 |
| Scenarios failing | 0 | 0 |
| Not implemented | 17 | 17 |

**No change expected.** S.3.1 and S.3.2 require pt-quote (T-002-02) to move from
NotImplemented to Pass. This ticket provides the type foundation they'll use.

## Acceptance Criteria Verification

### pt-project ✓
- [x] Project struct: id, scan_ref, zones, tiers (3), status, timestamps
- [x] Zone struct: id, geometry (geo::Polygon<f64>), zone_type enum, optional label
- [x] Tier struct: level (TierLevel), material assignments vec
- [x] MaterialAssignment struct: zone_id, material_id, optional overrides
- [x] ProjectStatus enum: draft, quoted, approved, complete
- [x] GeoJSON serialization round-trip (Project → GeoJSON → Project)
- [x] Tests for serialization, status transitions, zone CRUD operations

### pt-materials ✓
- [x] Material struct: id, name, category, unit, price_per_unit, depth_inches,
      texture_ref, photo_ref, supplier_sku, extrusion_behavior
- [x] MaterialCategory enum: hardscape, softscape, edging, fill
- [x] Unit enum: sq_ft, cu_yd, linear_ft, each
- [x] ExtrusionBehavior enum: SitsOnTop { height }, Fills { flush }, BuildsUp { height }
- [x] Serde serialization round-trip tests
- [x] Builder pattern with sensible defaults

## Quality Gate

- [x] `cargo test --workspace` — 60 tests, 0 failures
- [x] `cargo clippy --workspace -- -D warnings` — clean
- [x] `cargo run -p pt-scenarios` — no regressions

## Design Decisions Made

1. **Newtype IDs** (MaterialId, ZoneId) — prevents accidental swap at compile time
2. **pt-project depends on pt-materials** — honest dependency for MaterialId
3. **Separate GeoJSON conversion** — `to_geojson()`/`from_geojson()` methods,
   not serde default. GeoJSON is a domain format, not the default JSON shape.
4. **TierLevel enum + Vec<Tier>** — iterable for pt-quote, constructor enforces 3
5. **Tagged enum for ExtrusionBehavior** — `#[serde(tag = "type")]` for clean JSON

## Open Concerns

1. **Draft→Draft transition**: Currently `can_transition_to` allows Draft→Draft
   (the `(_, Draft)` arm). This is intentional (reset from any state) but means
   "transitioning" to the same state is a no-op that still updates `updated_at`.
   Acceptable — the alternative (special-casing same-state) adds complexity for
   no real benefit.

2. **GeoJSON precision**: f64 → JSON → f64 round-trips are exact for values that
   have clean f64 representations (our test coordinates). Exotic coordinates at
   the edge of f64 precision could drift. Not a practical concern for landscaping
   geometry (we're dealing in feet, not nanometers).

3. **Tier count enforcement**: `Project::new()` creates 3 tiers, but nothing
   prevents deserialization of a Project with != 3 tiers via normal JSON serde.
   `from_geojson()` also trusts the tier data. A validation method could be added
   if this becomes a real vector for bugs.

## Unblocked Work

T-002-02 (pt-quote) can now proceed — it has the types it needs from both crates.
