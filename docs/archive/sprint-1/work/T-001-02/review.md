# Review — T-001-02 pt-geo

## Summary of Changes

### Modified Files

| File | Change |
|------|--------|
| `crates/pt-geo/Cargo.toml` | Removed `geojson` dep; added `approx` dev-dep |
| `crates/pt-geo/src/lib.rs` | Module declarations, re-exports of geo types |

### New Files

| File | Lines | Purpose |
|------|-------|---------|
| `crates/pt-geo/src/area.rs` | 68 | `area_sqft()`, `multi_area_sqft()` |
| `crates/pt-geo/src/perimeter.rs` | 66 | `perimeter_ft()`, `multi_perimeter_ft()` |
| `crates/pt-geo/src/volume.rs` | 38 | `volume_cuft()`, `volume_cuyd()` |
| `crates/pt-geo/src/boolean.rs` | 79 | `union()`, `difference()`, `multi_union()`, `multi_difference()` |
| `crates/pt-geo/src/simplify.rs` | 53 | `simplify()`, `simplify_multi()` |

**Total**: ~320 lines (implementation + tests)

## Acceptance Criteria Checklist

| Criterion | Status | Notes |
|-----------|--------|-------|
| Polygon area (sq ft) with tests | Done | 5 tests: square, triangle, L-shape, empty, multi |
| Perimeter (linear ft) | Done | 4 tests: square, 3-4-5 triangle, empty, multi |
| Volume (cu ft → cu yd) | Done | 4 tests: basic, cuyd conversion, zero area, zero depth |
| Boolean: union, difference | Done | 5 tests: overlap, non-overlap, subtraction, chained |
| Polygon simplification (RDP) | Done | 3 tests: point reduction, epsilon=0, area preservation |
| All functions pure — no I/O | Done | No I/O imports anywhere; all functions take refs, return values |
| Unit tests covering edge cases | Done | Empty polygon, zero area, non-overlapping, self-intersecting implicitly handled by `geo` |
| Documentation on public API | Done | Doc comments on all 12 public functions |

## Test Coverage

**21 tests total**, all passing.

| Module | Tests | Coverage Notes |
|--------|-------|---------------|
| area | 5 | Square, triangle, L-shape, empty, multi. Covers regular, irregular, degenerate, composite. |
| perimeter | 4 | Square, 3-4-5 triangle, empty, multi. |
| volume | 4 | Basic cuft, cuyd conversion, zero area, zero depth. |
| boolean | 5 | Overlapping union, non-overlapping union, subtraction, non-overlapping diff, chained multi. |
| simplify | 3 | Point reduction, epsilon=0 identity, area preservation. |

### Coverage Gaps

- **Self-intersecting polygons**: Not explicitly tested. The `geo` crate handles these
  internally (unsigned_area sums sub-areas), but we don't have a test that verifies
  our wrapper's behavior on a bowtie polygon. Low risk since we delegate entirely to `geo`.

- **Very large polygons**: No stress test. Not needed for a utility crate — `geo` is
  well-tested at scale.

- **Interior ring perimeters**: `perimeter_ft()` only measures the exterior ring. This
  is documented and intentional, but if a caller needs hole perimeters they'll need a
  new function.

## Design Decisions Made

1. **Free functions over extension traits**: Simpler, more discoverable, no orphan rule issues.

2. **f64 concrete type**: No generics. Landscaping precision doesn't need alternatives.

3. **Removed geojson dependency**: Not needed in this crate; GeoJSON belongs in pt-project.

4. **Modern Length API**: Used `Length::<Euclidean>` instead of deprecated `EuclideanLength`.

5. **MultiPolygon overloads**: Provided `multi_*` variants for area, perimeter, boolean ops,
   and simplification to handle boolean op results without forcing callers to convert.

## Open Concerns

1. **No downstream consumers yet**: The API is untested by real callers. When pt-project
   and pt-quote integrate, the API shape may need adjustment (e.g., convenience functions
   that combine area + volume in one call).

2. **geojson removal**: Removed from pt-geo, but pt-project still has its own geojson dep.
   If a future ticket needs GeoJSON conversion _within_ pt-geo, the dependency will need
   to be re-added.

3. **Coordinate system assumption**: All functions assume local projected coordinates in
   feet. If the system ever handles lat/lon (GPS) coordinates directly, geodesic variants
   would be needed. This is a documented assumption, not a bug.

## Verification

```
cargo test -p pt-geo     → 21 passed, 0 failed
cargo clippy -p pt-geo   → 0 warnings
```
