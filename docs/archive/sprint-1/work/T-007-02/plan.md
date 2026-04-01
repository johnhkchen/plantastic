# T-007-02 Plan: Zone API Persistence + Live Measurements

## Step 1: Add measurements to ZoneResponse (backend)

- Add `pt-geo` dependency to `crates/plantastic-api/Cargo.toml`.
- In `zones.rs`: add `area_sqft: f64` and `perimeter_ft: f64` to `ZoneResponse`.
- In `zone_row_to_response()`: compute both values from `r.geometry` using pt-geo.
- Verify: `cargo build -p plantastic-api` compiles.

## Step 2: Update backend integration tests

- In `crud_test.rs`: after creating a zone with known geometry, assert that the response includes correct `area_sqft` and `perimeter_ft`.
- Use a simple rectangle (e.g., 10x20) so expected values are trivial to compute independently.
- Verify: `cargo test -p plantastic-api` passes (or tests are #[ignore] for Postgres).

## Step 3: Create frontend API types

- Create `web/src/lib/api/types.ts` with `ApiZone`, `GeoJsonPolygon` interfaces.
- Write `editorZoneToGeoJson()`: converts `EditorZone` vertices to GeoJSON Polygon (closes ring).
- Write `apiZoneToEditorZone()`: converts `ApiZone` geometry coords back to `EditorZone` vertices.
- Verify: TypeScript compiles.

## Step 4: Create zone API helpers

- Create `web/src/lib/api/zones.ts` with `fetchZones()`, `saveZones()`, `deleteZone()`.
- `saveZones()` calls PUT /projects/:id/zones with bulk body, then re-fetches to get computed measurements.
- Verify: TypeScript compiles.

## Step 5: Update mock API

- Add zone mutation handlers to `mock.ts`: PUT (bulk replace), POST (add), PATCH (update), DELETE.
- Mock responses include `area_sqft` and `perimeter_ft` computed from vertices (simple shoelace for area, sum of edge lengths for perimeter — just for mock plausibility).
- Verify: mock API handles all zone operations.

## Step 6: Wire editor page to API

- Rewrite `editor/+page.svelte`:
  - On mount: call `fetchZones(projectId)`, convert to `EditorZone[]`, set canvas state.
  - Track `apiZones: ApiZone[]` for measurements.
  - Watch zones array: on change, debounce 1.5s, call `saveZones()`, update `apiZones`.
  - Display zone info panel: for each zone, show name, type, area (sq ft), perimeter (ft).
  - Loading spinner on initial fetch. Error toast on save failure.
- Verify: in mock mode, zones persist across page navigations (mock is in-memory).

## Step 7: Implement S.2.1 scenario test

- In `design.rs`, implement `s_2_1_zone_drawing()`:
  - Construct 3 zones with known geometry (same as S.3.1 for consistency):
    - 12x15 ft patio: area = 180.0 sq ft, perimeter = 54.0 ft.
    - 8x20 ft bed: area = 160.0 sq ft, perimeter = 56.0 ft.
    - 10x10 edging square: area = 100.0 sq ft, perimeter = 40.0 ft.
  - Call `pt_geo::area::area_sqft()` and `pt_geo::perimeter::perimeter_ft()` on each.
  - Assert results match hand-computed values.
  - Return `ScenarioOutcome::Pass(Integration::TwoStar)`.
- Verify: `cargo run -p pt-scenarios` shows S.2.1 passing at TwoStar.

## Step 8: Claim milestones

- In `progress.rs`:
  - Claim "Zone editor: polygon drawing on plan view" for T-007-02.
  - Claim "pt-geo: area, perimeter, volume computation" for T-001-02 (backfill).
- Add notes describing what was delivered and what it enables.

## Step 9: Run quality gate

- `just check` — format + lint + test + scenarios.
- Fix any issues.

## Testing Strategy

| Layer | What | Where |
|-------|------|-------|
| Unit | area/perimeter already tested in pt-geo | `crates/pt-geo/src/area.rs`, `perimeter.rs` |
| Integration | Zone CRUD with measurements in response | `crates/plantastic-api/tests/crud_test.rs` |
| Scenario | S.2.1 zone drawing + measurements | `tests/scenarios/src/suites/design.rs` |
| Manual | Frontend zone persistence + display | Browser with mock API |

## Commit Plan

1. Backend: measurements in ZoneResponse + updated tests.
2. Frontend: types, API helpers, mock updates, editor page wiring.
3. Scenarios + milestones.
