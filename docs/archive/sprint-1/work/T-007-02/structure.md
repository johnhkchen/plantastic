# T-007-02 Structure: Zone API Persistence + Live Measurements

## Backend Changes

### Modified: `crates/plantastic-api/src/routes/zones.rs`

- Add `area_sqft: f64` and `perimeter_ft: f64` fields to `ZoneResponse`.
- Add `use pt_geo::area::area_sqft` and `use pt_geo::perimeter::perimeter_ft`.
- Update `zone_row_to_response()` to compute and populate the two new fields from `r.geometry`.
- No new routes. No handler logic changes. Just response enrichment.

### Modified: `crates/plantastic-api/Cargo.toml`

- Add `pt-geo` dependency (path = "../../crates/pt-geo").

### Modified: `crates/plantastic-api/tests/crud_test.rs`

- Existing zone_crud test: add assertions that the zone response includes `area_sqft` and `perimeter_ft` with correct values computed from the known test geometry.

## Frontend Changes

### New: `web/src/lib/api/types.ts`

API response types shared across features:
- `ApiZone` — maps to backend `ZoneResponse` (id, project_id, geometry, zone_type, label, sort_order, area_sqft, perimeter_ft, timestamps).
- `GeoJsonPolygon` — `{ type: "Polygon", coordinates: number[][][] }`.
- Helper functions: `editorZoneToGeoJson(zone: EditorZone) -> GeoJsonPolygon` and `apiZoneToEditorZone(zone: ApiZone) -> EditorZone`.

### New: `web/src/lib/api/zones.ts`

Zone-specific API functions:
- `fetchZones(projectId: string): Promise<ApiZone[]>` — GET /projects/:id/zones.
- `saveZones(projectId: string, zones: EditorZone[]): Promise<ApiZone[]>` — PUT /projects/:id/zones (bulk), returns full zone list after save (re-fetches to get computed fields).
- `deleteZone(projectId: string, zoneId: string): Promise<void>` — DELETE (used if individual delete needed).

### Modified: `web/src/routes/(app)/project/[id]/editor/+page.svelte`

Major rework of the editor page:
- On mount: fetch zones from API, convert to EditorZone[], populate canvas.
- Watch `zones` array: on change, debounce 1.5s, then bulk PUT to API.
- Store the latest `ApiZone[]` response for measurements display.
- Display area and perimeter badges next to each zone (in a sidebar or overlay panel).
- Loading and error states.

### Modified: `web/src/lib/api/mock.ts`

- Add PUT /projects/:id/zones handler (bulk update mock).
- Add POST /projects/:id/zones handler.
- Add PATCH/DELETE handlers for zones.
- Add `area_sqft` and `perimeter_ft` to mock zone responses (computed from vertices).

### Modified: `web/src/lib/components/zone-editor/ZoneEditor.svelte`

- No structural change to the editor itself. The parent page handles persistence.
- Potentially add an optional `measurements` prop for displaying area/perimeter overlays on the canvas, OR the parent page displays measurements in a separate panel.

**Decision:** Measurements display in the parent page as a zone info panel (cleaner separation), not overlaid on the canvas. The canvas remains a pure drawing tool.

## Scenario Changes

### Modified: `tests/scenarios/src/suites/design.rs`

- Implement `s_2_1_zone_drawing()`:
  - Construct zones with known geometry (same pattern as S.3.1).
  - Call `pt_geo::area::area_sqft()` and `pt_geo::perimeter::perimeter_ft()` on those polygons.
  - Assert results match independently computed values.
  - Return `ScenarioOutcome::Pass(Integration::TwoStar)`.

### Modified: `tests/scenarios/src/progress.rs`

- Claim "Zone editor: polygon drawing on plan view" milestone for T-007-02.
- Claim "pt-geo: area, perimeter, volume computation" milestone for T-001-02 (backfill).

## File Summary

| Action   | File |
|----------|------|
| Modified | `crates/plantastic-api/src/routes/zones.rs` |
| Modified | `crates/plantastic-api/Cargo.toml` |
| Modified | `crates/plantastic-api/tests/crud_test.rs` |
| New      | `web/src/lib/api/types.ts` |
| New      | `web/src/lib/api/zones.ts` |
| Modified | `web/src/routes/(app)/project/[id]/editor/+page.svelte` |
| Modified | `web/src/lib/api/mock.ts` |
| Modified | `tests/scenarios/src/suites/design.rs` |
| Modified | `tests/scenarios/src/progress.rs` |

## Module Boundaries

- pt-geo is a dependency of plantastic-api (new edge). Pure computation, no coupling concerns.
- Frontend API types (`types.ts`) are shared across features — this is where we standardize API response shapes.
- Zone API functions (`zones.ts`) isolate HTTP concerns from UI components.
- The editor component stays UI-only; the page coordinates persistence.

## Ordering

1. Backend first: add measurements to ZoneResponse, update tests.
2. Frontend types and API helpers.
3. Mock API updates.
4. Editor page wiring (load + auto-save + display).
5. Scenario test + milestone claims.
