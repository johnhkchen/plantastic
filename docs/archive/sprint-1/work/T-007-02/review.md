# T-007-02 Review: Zone API Persistence + Live Measurements

## Summary of Changes

This ticket wires the polygon drawing component (T-007-01) to the zone CRUD API (T-004-02), adds server-computed measurements to zone responses, and builds the frontend persistence layer with a measurements display panel.

## Files Changed

### Backend (Rust)

| Action   | File | Change |
|----------|------|--------|
| Modified | `crates/plantastic-api/Cargo.toml` | Added `pt-geo` dependency |
| Modified | `crates/plantastic-api/src/routes/zones.rs` | Added `area_sqft`, `perimeter_ft` to `ZoneResponse`; compute in `zone_row_to_response()` via pt-geo |
| Modified | `crates/plantastic-api/tests/crud_test.rs` | Added measurement assertions to `zone_crud` test (12x15 polygon → 180 sq ft, 54 ft perimeter) |

### Frontend (SvelteKit)

| Action   | File | Change |
|----------|------|--------|
| New      | `web/src/lib/api/types.ts` | `ApiZone`, `GeoJsonPolygon` types; `editorZoneToGeoJson()`, `apiZoneToEditorZone()` converters |
| New      | `web/src/lib/api/zones.ts` | `fetchZones()`, `saveZones()`, `deleteZone()` API helpers |
| Modified | `web/src/lib/api/mock.ts` | Full zone mutation handlers (PUT/POST/PATCH/DELETE) with mock area/perimeter computation |
| Modified | `web/src/routes/(app)/project/[id]/editor/+page.svelte` | Zone loading, debounced auto-save, measurements info panel |

### Scenarios + Milestones

| Action   | File | Change |
|----------|------|--------|
| Modified | `tests/scenarios/src/suites/design.rs` | Implemented S.2.1 with 4 polygon tests (3 rectangles + L-shape), TwoStar |
| Modified | `tests/scenarios/src/progress.rs` | Claimed zone editor milestone (T-007-02) and pt-geo milestone (T-001-02 backfill) |

## Scenario Dashboard — Before & After

| Metric | Before | After | Delta |
|--------|--------|-------|-------|
| Effective savings | 12.0 min (5.0%) | 20.0 min (8.3%) | +8.0 min (+3.3%) |
| Raw passing | 60.0 min | 80.0 min | +20.0 min |
| Passing scenarios | 3 | 4 | +1 (S.2.1) |
| Milestones | 4/19 | 6/19 | +2 |

### Scenarios Advanced

- **S.2.1** (Zone drawing with measurements): `NotImplemented` → `PASS ★★☆☆☆` (+8.0 effective min)
  - TwoStar because: the API route exists and returns computed measurements; the frontend loads zones from API and displays measurements. No polish/error handling that would justify ThreeStar.

### Milestones Claimed

- **Zone editor: polygon drawing on plan view** (T-007-02): canvas polygon component + API wiring + measurements panel.
- **pt-geo: area, perimeter, volume computation** (T-001-02 backfill): previously delivered but unclaimed.

## Test Coverage

| Layer | Tests | Status |
|-------|-------|--------|
| pt-geo unit tests | 21 tests (area, perimeter, volume, boolean, simplify) | All passing |
| S.2.1 scenario | 4 polygons: 12x15, 8x20, 10x10, L-shape | Passing at TwoStar |
| Backend integration | `zone_crud` now asserts `area_sqft` and `perimeter_ft` | Compiles; requires Postgres to run |
| Frontend TypeScript | `svelte-check` — 0 errors, 0 warnings | Passing |
| Quality gate | `just check` — fmt, lint, test, scenarios | All green |

### Coverage Gaps

1. **No frontend unit tests** for `editorZoneToGeoJson()` / `apiZoneToEditorZone()` conversion functions. Vitest is not configured in this project yet (noted in T-007-01 review). These are simple coordinate transforms but could benefit from tests once a frontend test runner exists.
2. **Integration test requires Postgres** — the `zone_crud` test with measurement assertions is `#[ignore]`. It will validate when run against a real database.
3. **No end-to-end test** of the full flow (draw polygon → auto-save → reload → measurements displayed). This would require browser automation (Playwright) which is not in scope.

## Acceptance Criteria Checklist

- [x] Zone save/update calls POST/PATCH /projects/:id/zones — bulk PUT via debounced auto-save
- [x] Zone delete calls DELETE /projects/:id/zones/:zid — available via `deleteZone()` API helper
- [x] API response includes computed area_sqft and perimeter_ft for each zone
- [x] Frontend displays area and perimeter next to each zone — measurements info panel
- [x] Zones reload correctly when the project page is refreshed — fetchZones on mount
- [x] S.2.1 scenario test registered and passing at ★★ — TwoStar with 4 polygon assertions
- [x] Claim milestone: "Zone editor: polygon drawing on plan view" in progress.rs

## Open Concerns

1. **Coordinate system assumption**: The frontend uses canvas pixel coordinates (1px ≈ 1ft). This is fine for V1 but will need a real coordinate mapping when LiDAR scans are integrated. The GeoJSON conversion is transparent to this — coordinates pass through as-is.

2. **Bulk PUT replaces all zone IDs**: After auto-save, zone IDs change (server generates new UUIDs). The editor re-syncs from the API response, which means any tier assignments referencing old zone IDs would break. This is a known issue from T-004-02 review. Mitigation: tier assignment UI (T-009-01) will need to handle this, or we should switch to individual POST/PATCH per zone instead of bulk replace.

3. **Auto-save re-sync can cause flicker**: When the editor re-syncs zones from the API response after a save, the canvas briefly redraws. This is cosmetic and acceptable for TwoStar but should be smoothed out for ThreeStar+ by matching zones by sort_order rather than replacing the entire array.

4. **Mock API uses approximate geometry**: The mock shoelace/distance formulas compute area and perimeter correctly for simple polygons, but may diverge slightly from pt-geo for complex shapes. This only affects mock mode development.
