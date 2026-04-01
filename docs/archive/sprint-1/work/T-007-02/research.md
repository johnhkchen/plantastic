# T-007-02 Research: Zone API Persistence + Live Measurements

## Ticket Goal

Wire the polygon drawing component (T-007-01) to the zone CRUD API (T-004-02). When zones are drawn or edited, they persist to the database. The API returns computed area/perimeter from pt-geo so the frontend displays measurements without duplicating geometry math in JS. Target: S.2.1 at TwoStar.

## Dependencies — What Exists

### T-007-01: Canvas Polygon Drawing (done)

- `web/src/lib/components/zone-editor/ZoneEditor.svelte` — 338 lines, Svelte 5 runes, raw HTML5 Canvas.
- Zone data: `EditorZone { id: string, vertices: Point[], zoneType: ZoneType, label: string }`.
- Vertices are in **canvas pixel coordinates** (not feet, not lat/lng).
- The editor page (`routes/(app)/project/[id]/editor/+page.svelte`) currently has a local `zones` state array with no API calls.
- The editor emits zone data via `bind:zones` — the parent page owns the array.
- No area/perimeter display exists yet.

### T-004-02: CRUD Routes (done)

- Zone routes in `crates/plantastic-api/src/routes/zones.rs`:
  - `GET /projects/{id}/zones` — returns `Vec<ZoneResponse>`
  - `POST /projects/{id}/zones` — returns 201 + `ZoneResponse`
  - `PUT /projects/{id}/zones` — bulk replace, returns `Vec<Uuid>`
  - `PATCH /projects/{id}/zones/{zid}` — returns 204
  - `DELETE /projects/{id}/zones/{zid}` — returns 204
- `ZoneResponse` fields: `id, project_id, geometry (GeoJSON), zone_type, label, sort_order, created_at, updated_at`.
- **No area/perimeter fields** in the current response. This is the key gap.
- Geometry stored as `GEOMETRY(POLYGON, 4326)` in PostGIS (EPSG:4326 = lat/lng).
- Zone type: `bed | patio | path | lawn | wall | edging`.

### pt-geo Crate (T-001-02, done)

- `area_sqft(polygon: &Polygon<f64>) -> f64` — unsigned area.
- `perimeter_ft(polygon: &Polygon<f64>) -> f64` — exterior ring Euclidean perimeter.
- Pure functions, no I/O. Already used by pt-quote for S.3.1/S.3.2.
- **Important**: These compute in coordinate units directly. The polygon coordinates in pt-project zones are in **feet** (plan-view local coordinates), not geographic degrees.

### Repository Layer (pt-repo)

- `pt_repo::zone::ZoneRow` — includes `geometry: Polygon<f64>`.
- The API route handler `zone_row_to_response()` converts `Polygon<f64>` to GeoJSON `serde_json::Value`.
- Conversion utilities in `pt_repo/src/convert.rs`: `polygon_to_geojson_string`, `geojson_string_to_polygon`.

### Frontend API Client

- `web/src/lib/api/client.ts` — `apiFetch<T>(path, options)` generic fetch wrapper.
- Auth: sends `X-Tenant-Id` and `Authorization` headers from session store.
- Mock API (`web/src/lib/api/mock.ts`) — has `GET /projects/:id/zones` returning mock data; missing POST/PUT/PATCH/DELETE handlers for zones.
- Proxy: `VITE_MOCK_API=true` toggles between mock and real.

### Scenario Harness

- `tests/scenarios/src/suites/design.rs` — S.2.1 is currently `ScenarioOutcome::NotImplemented`.
- S.2.1 needs to validate: polygon → zone stored → area/perimeter computed correctly.
- Target: `ScenarioOutcome::Pass(Integration::TwoStar)` — API computes and returns measurements.
- Pattern to follow: S.3.1 in `suites/quoting.rs` builds zones with known geometry and asserts independently computed values.

### Milestones

- "Zone editor: polygon drawing on plan view" — unlocks S.2.1, currently unclaimed.
- "pt-geo: area, perimeter, volume computation" — unlocks S.2.1, S.3.1, S.3.2, currently unclaimed.

## Coordinate System Gap

The frontend `EditorZone` uses canvas pixel coordinates. The backend expects GeoJSON polygons in some coordinate space. For V1 (no real LiDAR scan), the canvas coordinates ARE the coordinate system — 1 pixel = 1 foot is a reasonable mapping. The frontend needs to convert `Point[]` to GeoJSON Polygon and back. This is a straightforward transform: `[{x, y}]` → `{ type: "Polygon", coordinates: [[[x1,y1], [x2,y2], ...]] }`.

## Store vs Editor Zone Types

- `project.svelte.ts` defines `Zone { id, vertices: {x,y}[], zoneType, label }`.
- `zone-editor/types.ts` defines `EditorZone { id, vertices: Point[], zoneType, label }`.
- These are structurally identical. The project store `Zone` type doesn't include `area_sqft` or `perimeter_ft` — those need to be added for display.

## Key Observations

1. The API returns GeoJSON geometry but no computed measurements. Adding `area_sqft` and `perimeter_ft` to `ZoneResponse` requires calling pt-geo in the route handler.
2. The frontend editor page is completely disconnected from the API — no load-on-mount, no save-on-edit.
3. The mock API has zone listing but not zone mutation routes.
4. The existing S.2.1 scenario is a stub. It can test pure computation (area/perimeter from known polygons) at OneStar, but to reach TwoStar the test should verify the API layer returns correct measurements.
5. Two milestones can be claimed: zone editor + pt-geo (though pt-geo was already delivered by T-001-02, it's unclaimed in progress.rs).
