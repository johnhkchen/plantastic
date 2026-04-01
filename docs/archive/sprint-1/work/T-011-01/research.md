# T-011-01 Research: pt-satellite crate

## Ticket Summary

Address → geocoded lat/lng → lot boundary + tree canopy + sun exposure baseline.
Output: `ProjectBaseline` struct. S.1.2 scenario at ★☆☆☆☆. Claim milestone.

## Dependency Crates

### pt-solar (T-010-01, delivered)

Location: `crates/pt-solar/`

Key types for pt-satellite:
- `Coordinates { latitude: f64, longitude: f64 }` — WGS84 lat/lng
- `LatLngBounds { south, west, north, east: f64 }` — bounding box
- `GridConfig { resolution_meters: f64, sample_days: u32 }` — grid params
- `ExposureGrid { bounds, resolution_meters, width, height, values: Vec<f32>, sample_days_used }` — result
- `METERS_PER_DEGREE_LAT: f64 = 111_320.0`

Key function: `radiance_grid(bounds: &LatLngBounds, date_range: (NaiveDate, NaiveDate), config: &GridConfig) -> ExposureGrid`

Pure computation, no I/O. ~2ms for a full year at a single point.

### pt-climate (T-010-02, delivered)

Location: `crates/pt-climate/`

Has its own `Coordinates` type (identical structure to pt-solar's but different Rust type).
Provides `climate_profile()` convenience function. Not directly required by acceptance criteria
but the spec mentions climate enrichment in the baseline pipeline.

### pt-geo (T-001-02, delivered)

Location: `crates/pt-geo/`

Re-exports: `geo::{Coord, LineString, MultiPolygon, Polygon, coord, polygon}`
Functions: `area_sqft()`, `perimeter_ft()`, etc.

pt-satellite needs `Polygon<f64>` for lot boundaries.

### pt-project

Location: `crates/pt-project/`

Defines `Project` aggregate root. The `baseline` field in the DB schema is `JSONB`:
```sql
baseline JSONB,  -- {lot_polygon, trees[], sun_grid_key}
```

The spec dependency graph shows pt-satellite depends on pt-project (→ pt-geo).
However, pt-project's `Project` struct currently has no `baseline` field — only
`scan_ref: Option<String>`. The baseline type should be defined in pt-satellite
and referenced by pt-project/pt-repo later. For this ticket, pt-satellite is
self-contained.

## Existing Patterns

### Crate structure convention
- `lib.rs` with module declarations and public re-exports
- Internal modules: `types.rs`, `error.rs`, feature modules
- No `mod.rs` files except for suite aggregation
- `thiserror` for error enums
- All public types derive `Serialize, Deserialize`
- `#[serde(rename_all = "snake_case")]` on enums
- Newtype wrappers over `Uuid` for IDs

### Cargo.toml convention
- `edition.workspace = true`, `license.workspace = true`, `rust-version.workspace = true`
- Dependencies reference workspace: `chrono.workspace = true`
- Dev deps include `pt-test-utils` and `approx`
- `[lints] workspace = true`

### Testing convention
- `#[cfg(test)] mod tests` blocks in source files
- `pt_test_utils::timed()` wrapping test bodies
- `approx::assert_relative_eq!` for float comparisons
- Independent expected-value computation (no calling system code to derive expected)

### Scenario convention
- Static `SCENARIOS` array in suite file
- Test functions return `ScenarioOutcome` enum
- S.1.3 is the closest reference: constructs bounds, date range, config → calls `radiance_grid` → validates output
- OneStar = pure computation, no API, no UI

## Data Sources (Production vs OneStar)

The spec says: "I/O lives at the edges: apps/api for HTTP, pt-scan for file processing,
pt-satellite for external data fetches."

For **production**, pt-satellite will fetch:
1. Geocoding: address → lat/lng (external geocoding API)
2. Parcel data: lat/lng → lot boundary polygon (SF gov parcel API or cached dataset)
3. Canopy data: lot bounds → tree positions + heights (Meta canopy height dataset)

For **OneStar** (this ticket), all data is embedded:
- Known SF test addresses with hardcoded lat/lng
- Embedded parcel polygon for test addresses
- Embedded canopy data (tree positions, heights, spreads)
- Real pt-solar computation for sun grid (not mocked)

The design must support both modes via a data source abstraction (trait).

## Database Schema Context

The `projects` table has:
```sql
baseline JSONB,  -- {lot_polygon, trees[], sun_grid_key}
```

This means `ProjectBaseline` must serialize to JSON matching this shape.
The `sun_grid_key` suggests the full `ExposureGrid` is stored separately (S3 or similar),
with only a reference key in the baseline. For OneStar, we can include the grid inline.

## Scenario S.1.2

Currently `NotImplemented` in `tests/scenarios/src/suites/site_assessment.rs` (line 48).
Time savings: 25.0 minutes. Replaces cold-start research on new sites.

The test function needs to:
1. Construct a known SF address
2. Call pt-satellite to produce a `ProjectBaseline`
3. Assert lot polygon has reasonable area for an SF residential lot
4. Assert detected trees have plausible heights/spreads
5. Assert sun grid covers the lot bounds with valid values
6. Return `ScenarioOutcome::Pass(Integration::OneStar)`

## Milestone

`progress.rs` line 47: "pt-satellite: address → lot + canopy + sun baseline"
Currently `delivered_by: None`. Must set to `Some("T-011-01")` with implementation note.

## Constraints and Risks

1. **Two `Coordinates` types**: pt-solar and pt-climate each define their own. pt-satellite
   should re-export from pt-solar (its primary dependency) and provide `From` impls if
   pt-climate integration is needed.

2. **No real I/O for OneStar**: Embedded data only. Must be clearly separated so real
   data sources can be swapped in without changing the core logic.

3. **Polygon coordinate system**: pt-geo uses `Polygon<f64>` where coords are in feet
   (for zone geometry). Lot boundary polygons from parcel data are in WGS84 (lat/lng).
   These are different coordinate spaces — the lot boundary stays in WGS84 for
   geo-referencing; zone polygons are in a local coordinate system.

4. **ExposureGrid ownership**: The grid is computed by pt-solar. pt-satellite calls
   `radiance_grid()` with the lot bounds and returns the result. No duplication of
   solar logic.

5. **Test data realism**: Embedded test data should represent a real SF residential
   property (e.g., ~5,000-10,000 sq ft lot, 2-5 trees, typical for Inner Sunset or
   similar neighborhood).
