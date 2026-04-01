# T-007-02 Progress: Zone API Persistence + Live Measurements

## Completed

### Step 1: Backend — measurements in ZoneResponse
- Added `pt-geo` dependency to `plantastic-api` Cargo.toml.
- Added `area_sqft: f64` and `perimeter_ft: f64` to `ZoneResponse` struct.
- Imported `pt_geo::area::area_sqft` and `pt_geo::perimeter::perimeter_ft`.
- Updated `zone_row_to_response()` to compute both measurements from `r.geometry`.
- `cargo check -p plantastic-api` passes.

### Step 2: Backend integration test update
- Added assertions to `zone_crud` test: after creating a 12x15 zone, verify `area_sqft ≈ 180.0` and `perimeter_ft ≈ 54.0`.
- Test is `#[ignore]` (requires Postgres), but compiles and will validate when run against real DB.

### Step 3: Frontend API types
- Created `web/src/lib/api/types.ts` with `ApiZone`, `GeoJsonPolygon` interfaces.
- `editorZoneToGeoJson()`: converts canvas vertices to GeoJSON Polygon (closes ring).
- `apiZoneToEditorZone()`: converts GeoJSON coordinates back to canvas vertices (strips closing coord).

### Step 4: Zone API helpers
- Created `web/src/lib/api/zones.ts` with `fetchZones()`, `saveZones()`, `deleteZone()`.
- `saveZones()` bulk PUTs then re-fetches to get server-computed measurements.

### Step 5: Mock API updates
- Rewrote mock zone storage to use `MockApiZone` objects with full API shape.
- Added PUT (bulk replace), POST (add), PATCH (update), DELETE handlers.
- Mock computes `area_sqft` and `perimeter_ft` from geometry using shoelace/distance formulas.

### Step 6: Editor page wiring
- Rewrote `editor/+page.svelte` with:
  - Fetch zones on mount, convert to EditorZone[] for canvas.
  - Debounced auto-save (1.5s) via bulk PUT on zone changes.
  - Zone info panel (right sidebar) showing name, type, area (sq ft), perimeter (ft).
  - Loading, saving, error, and saved states.
- TypeScript compilation: 0 errors, 0 warnings.

### Step 7: S.2.1 scenario test
- Implemented `s_2_1_zone_drawing()` in `design.rs`.
- Tests 3 rectangular zones + 1 L-shaped irregular polygon.
- All measurements match hand-computed values.
- Returns `ScenarioOutcome::Pass(Integration::TwoStar)`.

### Step 8: Milestone claims
- Claimed "Zone editor: polygon drawing on plan view" for T-007-02.
- Claimed "pt-geo: area, perimeter, volume computation" for T-001-02 (backfill).

### Step 9: Quality gate
- `just check` passes: format, lint, test, scenarios all green.

## Deviations from Plan

None. All steps executed as planned.
