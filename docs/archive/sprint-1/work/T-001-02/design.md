# Design — T-001-02 pt-geo

## Problem

Provide a clean, domain-specific geometry API for Plantastic. The crate must wrap the
`geo` crate's generic traits into functions that speak in landscaping units (sq ft,
linear ft, cu yd) and handle the MultiPolygon reality of boolean operations.

## Options Considered

### Option A: Re-export geo traits directly

Callers `use geo::Area` and call `polygon.unsigned_area()` themselves.

- **Pro**: Zero code to write, zero maintenance
- **Con**: Leaks geo API everywhere; callers must know trait imports; no unit semantics;
  no domain-specific helpers (volume, cu yd conversion); changing geo version affects
  all callers; no central place for edge-case handling

**Rejected**: No encapsulation, no domain value.

### Option B: Newtype wrapper around Polygon/MultiPolygon

```rust
pub struct Zone(Polygon<f64>);
impl Zone { fn area_sqft(&self) -> f64 { ... } }
```

- **Pro**: Full control, domain naming
- **Con**: Imposes ownership semantics; callers must convert to/from Zone; duplicates
  pt-project's Zone concept; adds friction without clear benefit when the underlying
  type is already well-defined

**Rejected**: Over-abstraction for a utility crate. Zone semantics belong in pt-project.

### Option C: Free functions taking geo types as input ✓

```rust
pub fn area_sqft(polygon: &Polygon<f64>) -> f64 { ... }
pub fn perimeter_ft(polygon: &Polygon<f64>) -> f64 { ... }
pub fn volume_cuyd(area_sqft: f64, depth_ft: f64) -> f64 { ... }
pub fn union(a: &Polygon<f64>, b: &Polygon<f64>) -> MultiPolygon<f64> { ... }
pub fn simplify(polygon: &Polygon<f64>, epsilon: f64) -> Polygon<f64> { ... }
```

- **Pro**: Simple, composable, no wrapper types, domain-specific naming, callers work
  with standard geo types, easy to test, easy to extend
- **Con**: No method syntax (callers write `pt_geo::area_sqft(&poly)` not `poly.area_sqft()`)
- **Mitigation**: Method syntax is unnecessary for a utility crate; free functions are
  idiomatic Rust for stateless operations

**Selected**: Best balance of simplicity, domain clarity, and encapsulation.

### Option D: Extension traits on geo types

```rust
pub trait AreaExt { fn area_sqft(&self) -> f64; }
impl AreaExt for Polygon<f64> { ... }
```

- **Pro**: Method syntax, feels native
- **Con**: Trait imports are infectious; orphan rule may bite later; harder to discover;
  more boilerplate than free functions for the same result

**Rejected**: Marginal ergonomic gain doesn't justify the complexity for a small API.

## Chosen Approach: Free Functions (Option C)

### API Design

#### Area Module

```rust
pub fn area_sqft(polygon: &Polygon<f64>) -> f64
pub fn multi_area_sqft(mp: &MultiPolygon<f64>) -> f64
```

- Uses `unsigned_area()` — always positive, regardless of winding
- MultiPolygon variant for boolean op results

#### Perimeter Module

```rust
pub fn perimeter_ft(polygon: &Polygon<f64>) -> f64
pub fn multi_perimeter_ft(mp: &MultiPolygon<f64>) -> f64
```

- Calls `polygon.exterior().length::<Euclidean>()` (modern API, not deprecated)
- Exterior ring only — interior rings (holes) are separate perimeters if needed

#### Volume Module

```rust
pub fn volume_cuft(area_sqft: f64, depth_ft: f64) -> f64
pub fn volume_cuyd(area_sqft: f64, depth_ft: f64) -> f64
```

- Pure arithmetic: `area × depth` for cu ft, `÷ 27` for cu yd
- Takes area as input (not polygon) — decoupled from geometry

#### Boolean Operations Module

```rust
pub fn union(a: &Polygon<f64>, b: &Polygon<f64>) -> MultiPolygon<f64>
pub fn difference(a: &Polygon<f64>, b: &Polygon<f64>) -> MultiPolygon<f64>
```

- Returns `MultiPolygon` because that's the honest return type
- Only union and difference per acceptance criteria (intersection/xor can be added later)
- Also provide MultiPolygon overloads for chaining:

```rust
pub fn multi_union(a: &MultiPolygon<f64>, b: &Polygon<f64>) -> MultiPolygon<f64>
pub fn multi_difference(a: &MultiPolygon<f64>, b: &Polygon<f64>) -> MultiPolygon<f64>
```

#### Simplification Module

```rust
pub fn simplify(polygon: &Polygon<f64>, epsilon: f64) -> Polygon<f64>
pub fn simplify_multi(mp: &MultiPolygon<f64>, epsilon: f64) -> MultiPolygon<f64>
```

- Ramer-Douglas-Peucker (the `Simplify` trait), as specified in acceptance criteria
- Epsilon in coordinate units (feet)

### Re-exports

Re-export core geo types from `lib.rs` so callers don't need a direct `geo` dependency:

```rust
pub use geo::{Polygon, MultiPolygon, LineString, Coord, coord, polygon};
```

This centralizes the geo version dependency in pt-geo.

### Error Strategy

No `Result` types. Follow geo's pattern:
- Degenerate inputs produce defined outputs (0.0 for empty/zero-area)
- Boolean ops on degenerate inputs return empty MultiPolygon
- Document edge-case behavior in doc comments

### Testing Strategy

- Known-area polygons: 10×10 square = 100 sq ft, right triangle, L-shape
- Perimeter: same shapes with known perimeters
- Volume: arithmetic spot checks
- Boolean ops: overlapping squares (verify union area = expected), subtraction
- Simplification: complex polygon → simplified has fewer points, area within tolerance
- Edge cases: empty polygon, collinear points, self-intersecting

## Decisions

1. **f64 only** — no generic `<T: CoordFloat>`. Landscaping doesn't need f32 or decimal
   precision in geometry. Concrete types simplify the API and error messages.

2. **Exterior perimeter only** — interior ring perimeters are a separate concern. If needed
   later, add `perimeter_with_holes_ft()`.

3. **No GeoJSON in this ticket** — geojson dependency exists but conversion belongs in
   pt-project where Zone has semantic meaning. May remove the geojson dep later.

4. **Module-per-concern** — area.rs, perimeter.rs, volume.rs, boolean.rs, simplify.rs.
   Each module is small (~30 lines) but the separation aids discoverability.
