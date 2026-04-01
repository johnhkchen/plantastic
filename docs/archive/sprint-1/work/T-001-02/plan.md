# Plan — T-001-02 pt-geo

## Implementation Steps

### Step 1: Update Cargo.toml

- Remove `geojson` dependency (not needed in pt-geo)
- Add `approx = "0.5"` as dev-dependency for float comparison in tests
- Verify: `cargo check -p pt-geo` passes

### Step 2: Scaffold lib.rs with module declarations and re-exports

- Replace stub doc comment with crate-level docs
- Declare all 5 modules: area, perimeter, volume, boolean, simplify
- Add re-exports: `Polygon`, `MultiPolygon`, `LineString`, `Coord`, `coord!`, `polygon!`
- Verify: `cargo check -p pt-geo` (will fail until modules exist — that's fine, or
  create empty module files simultaneously)

### Step 3: Implement area.rs

- `area_sqft()` using `geo::Area::unsigned_area()`
- `multi_area_sqft()` using same trait on MultiPolygon
- Tests: 10×10 square, right triangle, L-shape, empty polygon, MultiPolygon
- Verify: `cargo test -p pt-geo -- area`

### Step 4: Implement perimeter.rs

- `perimeter_ft()` calling `polygon.exterior().euclidean_length()`
  (use `EuclideanLength` — the `Length::<Euclidean>` API requires importing the metric
   space type which adds complexity for no benefit here; deprecated warning is acceptable
   or we can use the new API if clean)
- `multi_perimeter_ft()` summing exterior perimeters
- Tests: 10×10 square (40.0), 3-4-5 triangle (12.0), MultiPolygon
- Verify: `cargo test -p pt-geo -- perimeter`

### Step 5: Implement volume.rs

- `volume_cuft()` = area × depth
- `volume_cuyd()` = cuft / 27.0
- Tests: known values, zero inputs
- Verify: `cargo test -p pt-geo -- volume`

### Step 6: Implement boolean.rs

- `union()`, `difference()` using `geo::BooleanOps` trait methods
- `multi_union()`, `multi_difference()` for MultiPolygon inputs
- Tests: overlapping squares, non-overlapping, subtraction
- Verify: `cargo test -p pt-geo -- boolean`

### Step 7: Implement simplify.rs

- `simplify()` using `geo::Simplify` trait
- `simplify_multi()` for MultiPolygon
- Tests: complex polygon simplifies, epsilon=0 identity, area preservation
- Verify: `cargo test -p pt-geo -- simplify`

### Step 8: Full verification

- `cargo test -p pt-geo` — all tests pass
- `cargo clippy -p pt-geo` — no warnings
- `cargo doc -p pt-geo --no-deps` — docs build clean
- Review public API matches structure.md

## Testing Strategy

### Unit Tests (inline per module)

Each module contains `#[cfg(test)] mod tests` with:

| Module | Test Cases |
|--------|-----------|
| area | square=100, triangle=12, L-shape, empty=0, multi |
| perimeter | square=40, 3-4-5 triangle=12, multi=sum |
| volume | cuft=area×depth, cuyd=cuft/27, zeros |
| boolean | overlap union < sum, difference = outer-inner, non-overlap |
| simplify | fewer points, epsilon=0 identity, area preserved |

### Float Comparison

Use `approx::assert_relative_eq!` with default epsilon for all float assertions.
This handles floating-point imprecision without brittle exact comparisons.

### Edge Cases

- Empty Polygon (`Polygon::new(LineString::new(vec![]), vec![])`) → area 0, perimeter 0
- Self-intersecting polygon (bowtie shape) → unsigned_area returns sum of sub-areas
- Zero-depth volume → 0.0

### No Integration Tests Needed

All functions are pure with no I/O. Unit tests are sufficient. Integration with
downstream crates (pt-project, pt-quote) will be tested when those crates are built.

## Commit Strategy

Single commit after all modules are implemented and tests pass. The changes are small
enough (~250 lines of code + tests) and tightly coupled — partial commits add no value.
