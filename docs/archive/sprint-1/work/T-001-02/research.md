# Research — T-001-02 pt-geo

## Ticket Summary

pt-geo is the geometry foundation crate for Plantastic. Every spatial operation flows through it:
area (quotes), perimeter (edging), volume (fill materials), boolean ops (zone editing),
simplification (display). Thin wrapper around the `geo` crate. Pure functions, no I/O.

## Codebase State

### Workspace Layout

```
Cargo.toml              # workspace root, resolver = "2", edition 2021
crates/
  pt-geo/               # THIS CRATE — currently empty stub (1 doc comment)
  pt-project/           # Zone/tier model, GeoJSON serialization (depends on geo, geojson)
  pt-materials/         # Material catalog, pricing (no geo dependency)
  pt-quote/             # Quote engine, quantity takeoff (no geo dependency yet)
```

### pt-geo Current State

- `Cargo.toml`: depends on `geo` (workspace = 0.29), `geojson` (0.24), `serde` (1)
- `src/lib.rs`: single doc comment, no code

### Downstream Consumers

No crate currently lists `pt-geo` as a dependency. Expected consumers:
- `pt-project` — will need area/perimeter for zone metadata, boolean ops for zone editing
- `pt-quote` — will need area/perimeter/volume for quantity takeoff

## geo Crate API Surface (v0.29.3)

### Core Types (from geo-types 0.7.18)

| Type | Role in Plantastic |
|------|-------------------|
| `Coord<f64>` | 2D coordinate, not a Geometry variant |
| `Polygon<f64>` | Primary: landscape zones, patios, lawn areas |
| `MultiPolygon<f64>` | Result of boolean ops; composite zones |
| `LineString<f64>` | Polygon rings; edge/path features |

Macros: `coord!`, `polygon!`, `linestring!`, `point!` for ergonomic construction.

### Area — `geo::Area` trait

- `signed_area(&self) -> f64` — negative if clockwise winding
- `unsigned_area(&self) -> f64` — absolute area
- Implemented on: `Polygon`, `MultiPolygon`, `LineString`, `Line`, `Point`
- Returns coordinate-unit² (if coords are in feet → sq ft)

For geodesic (lat/lon) area: `GeodesicArea` and `ChamberlainDuquetteArea` traits exist
but are unnecessary if we work in projected/local coordinates (feet), which is the plan.

### Perimeter — `geo::EuclideanLength` (deprecated) → `geo::Length`

- New API (v0.29): `line_string.length::<Euclidean>()` using metric-space generic
- Old API (deprecated): `euclidean_length(&self)`
- Implemented on: `Line`, `LineString`, `MultiLineString`
- For polygon perimeter: call on `polygon.exterior()`
- Returns coordinate-units (feet if coords are in feet)

### Boolean Operations — `geo::BooleanOps` trait

- Methods: `union()`, `difference()`, `intersection()`, `xor()`, `boolean_op(OpType)`
- All return `MultiPolygon<T>`
- Backed by `i_overlay` crate (polygon clipping)
- Constraint: `T: BoolOpsNum` (satisfied by `f64`)
- Implemented on: `Polygon<T>`, `MultiPolygon<T>`

### Simplification — `geo::Simplify` trait (Ramer-Douglas-Peucker)

- `simplify(&self, epsilon: &f64) -> Self`
- Implemented on: `Polygon`, `MultiPolygon`, `LineString`, `MultiLineString`
- Epsilon = max perpendicular distance threshold (in coordinate units)
- Also available: `SimplifyVw` (Visvalingam-Whyatt), `SimplifyVwPreserve` (topology-safe)

### Volume

No volume trait in `geo` — volume is `area × depth`. This is domain logic:
- Input: area (sq ft) + depth (ft)
- Output: cu ft, with conversion to cu yd (÷ 27)
- Pure arithmetic, no spatial dependency

## Constraints and Assumptions

1. **Coordinate system**: Local projected coordinates in feet (not lat/lon). This means
   planar `Area` and `Euclidean` length are correct; no geodesic math needed.

2. **f64 precision**: All geo operations use `f64`. Sufficient for landscaping scale
   (yard-level, not surveying-grade). No need for `f32` or decimal types in geometry.

3. **Pure functions**: Ticket explicitly requires no I/O, no side effects. The crate is
   a stateless utility library.

4. **geojson dependency**: Already in Cargo.toml. Likely needed for serialization of
   results, but the ticket doesn't mention GeoJSON. May be used by pt-project instead.
   Keep the dependency but don't build GeoJSON features in this ticket.

5. **Error handling**: geo trait methods return values directly (no Result). Edge cases
   (empty polygon, self-intersecting) produce defined results (0.0 area, etc.) rather
   than errors. Our wrapper can follow the same pattern for most functions, but boolean
   ops on degenerate inputs may need consideration.

6. **MultiPolygon returns**: Boolean ops always return `MultiPolygon`. Callers need to
   handle this. We should re-export or newtype as needed.

## Edge Cases to Address

- Self-intersecting polygons: `unsigned_area()` handles them (sum of signed sub-areas)
- Zero-area polygons (collinear points): returns 0.0
- Empty polygons (no coordinates): returns 0.0
- Boolean ops on non-overlapping polygons: union = both, difference = self unchanged
- Simplification with epsilon=0: returns original polygon unchanged
- Simplification that collapses a polygon: may return degenerate polygon (< 3 points)

## Dependencies Graph

```
pt-geo
  ├── geo 0.29.3 (polygon math, boolean ops via i_overlay)
  ├── geojson 0.24 (conversion traits, may defer usage)
  └── serde 1 (derive for any wrapper types)
```

No additional dependencies needed. The `geo` crate provides everything required.

## Key Observations

1. The wrapper is thin by design — we're not reimplementing algorithms, just providing
   a domain-specific API surface (sq ft, linear ft, cu yd) over geo's generic traits.

2. Unit conversions (cu ft → cu yd) are the main value-add beyond re-exporting traits.

3. The `MultiPolygon` return type from boolean ops is the biggest API design decision —
   callers might want a single Polygon back, but that's not always possible.

4. The `Length` trait's new metric-space generic pattern (`length::<Euclidean>()`) is the
   modern API; avoid the deprecated `euclidean_length()`.

5. No downstream consumers yet, so the API can be designed freely without migration concern.
