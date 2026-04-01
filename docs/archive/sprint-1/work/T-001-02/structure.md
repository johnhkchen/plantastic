# Structure — T-001-02 pt-geo

## File Changes

### Modified Files

#### `crates/pt-geo/Cargo.toml`

Add `[dev-dependencies]` for `approx` (float comparison in tests). Remove `geojson`
dependency — not needed for this crate's scope (GeoJSON conversion belongs in pt-project).

```toml
[dependencies]
geo.workspace = true
serde.workspace = true

[dev-dependencies]
approx = "0.5"
```

Decision: drop `geojson` from pt-geo. If pt-project needs it, pt-project already has its
own geojson dependency.

#### `crates/pt-geo/src/lib.rs`

Module declarations, re-exports, and crate-level docs.

```rust
//! Geometry and spatial math for Plantastic.
//!
//! Thin wrapper around the `geo` crate providing domain-specific functions
//! that operate in landscaping units (sq ft, linear ft, cu yd).
//!
//! All functions are pure — no I/O, no side effects.

pub mod area;
pub mod boolean;
pub mod perimeter;
pub mod simplify;
pub mod volume;

// Re-export core geo types so callers don't need a direct geo dependency.
pub use geo::{coord, polygon, Coord, LineString, MultiPolygon, Polygon};
```

### New Files

#### `crates/pt-geo/src/area.rs`

Public functions:
- `area_sqft(polygon: &Polygon<f64>) -> f64`
- `multi_area_sqft(mp: &MultiPolygon<f64>) -> f64`

Internal: imports `geo::Area` trait, calls `unsigned_area()`.

Tests (inline `#[cfg(test)]` module):
- 10×10 square → 100.0
- Right triangle (0,0)-(6,0)-(0,4) → 12.0
- L-shaped polygon (union of two rectangles) → known area
- Empty polygon → 0.0
- MultiPolygon with two non-overlapping squares

#### `crates/pt-geo/src/perimeter.rs`

Public functions:
- `perimeter_ft(polygon: &Polygon<f64>) -> f64`
- `multi_perimeter_ft(mp: &MultiPolygon<f64>) -> f64`

Internal: imports `geo::EuclideanLength` trait (or the new `Length` + `Euclidean` API),
calls on `polygon.exterior()`.

Tests:
- 10×10 square → 40.0
- 3-4-5 right triangle → 12.0
- MultiPolygon perimeter = sum of individual perimeters

#### `crates/pt-geo/src/volume.rs`

Public functions:
- `volume_cuft(area_sqft: f64, depth_ft: f64) -> f64`
- `volume_cuyd(area_sqft: f64, depth_ft: f64) -> f64`

Pure arithmetic. No geo imports.

Tests:
- 100 sqft × 0.5 ft = 50 cuft
- 50 cuft / 27 ≈ 1.8519 cuyd
- Zero area → 0.0
- Zero depth → 0.0

#### `crates/pt-geo/src/boolean.rs`

Public functions:
- `union(a: &Polygon<f64>, b: &Polygon<f64>) -> MultiPolygon<f64>`
- `difference(a: &Polygon<f64>, b: &Polygon<f64>) -> MultiPolygon<f64>`
- `multi_union(a: &MultiPolygon<f64>, b: &Polygon<f64>) -> MultiPolygon<f64>`
- `multi_difference(a: &MultiPolygon<f64>, b: &Polygon<f64>) -> MultiPolygon<f64>`

Internal: imports `geo::BooleanOps` trait.

Tests:
- Two overlapping 10×10 squares → union area < 200 (overlap removed)
- Subtract inner square from outer → area = outer - inner
- Non-overlapping union → combined area = sum
- Non-overlapping difference → unchanged

#### `crates/pt-geo/src/simplify.rs`

Public functions:
- `simplify(polygon: &Polygon<f64>, epsilon: f64) -> Polygon<f64>`
- `simplify_multi(mp: &MultiPolygon<f64>, epsilon: f64) -> MultiPolygon<f64>`

Internal: imports `geo::Simplify` trait.

Tests:
- Complex polygon simplifies to fewer points
- Epsilon = 0.0 → same number of points
- Area of simplified polygon ≈ original (within tolerance)

## Module Boundaries

```
lib.rs
  ├── area.rs          (geo::Area)
  ├── perimeter.rs     (geo::EuclideanLength)
  ├── volume.rs        (no geo dependency)
  ├── boolean.rs       (geo::BooleanOps)
  └── simplify.rs      (geo::Simplify)
```

Each module:
- Imports only the geo trait it needs
- Exposes only `pub fn` — no public types beyond re-exports in lib.rs
- Contains inline `#[cfg(test)] mod tests` — no separate test files
- Uses `f64` concretely, no generics

## Public API Summary

```
pt_geo::area::area_sqft(&Polygon<f64>) -> f64
pt_geo::area::multi_area_sqft(&MultiPolygon<f64>) -> f64
pt_geo::perimeter::perimeter_ft(&Polygon<f64>) -> f64
pt_geo::perimeter::multi_perimeter_ft(&MultiPolygon<f64>) -> f64
pt_geo::volume::volume_cuft(f64, f64) -> f64
pt_geo::volume::volume_cuyd(f64, f64) -> f64
pt_geo::boolean::union(&Polygon<f64>, &Polygon<f64>) -> MultiPolygon<f64>
pt_geo::boolean::difference(&Polygon<f64>, &Polygon<f64>) -> MultiPolygon<f64>
pt_geo::boolean::multi_union(&MultiPolygon<f64>, &Polygon<f64>) -> MultiPolygon<f64>
pt_geo::boolean::multi_difference(&MultiPolygon<f64>, &Polygon<f64>) -> MultiPolygon<f64>
pt_geo::simplify::simplify(&Polygon<f64>, f64) -> Polygon<f64>
pt_geo::simplify::simplify_multi(&MultiPolygon<f64>, f64) -> MultiPolygon<f64>
```

Re-exports at crate root:
```
pt_geo::Polygon, MultiPolygon, LineString, Coord, coord!, polygon!
```

## Dependency Changes

- **Add**: `approx = "0.5"` as dev-dependency (float assertions in tests)
- **Remove**: `geojson` from pt-geo dependencies (not needed here)
- **No workspace-level changes**: `approx` is only a dev-dep of pt-geo

## Ordering

No ordering constraints between modules — they are independent. All can be implemented
in any order. Tests within each module are self-contained.
