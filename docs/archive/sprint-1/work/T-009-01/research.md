# T-009-01 Research: Material Assignment UI

## Ticket Summary

Build the UI where a landscaper selects a zone, picks a material from the tenant catalog, and assigns it to a tier (good/better/best). Persist via `PUT /projects/:id/tiers/:tier`. Show live quote total via `GET /projects/:id/quote/:tier`.

## Dependencies (Both Complete)

### T-007-02: Zone API Persistence + Measurements
- Zone editor at `web/src/routes/(app)/project/[id]/editor/+page.svelte`
- Debounced auto-save via `PUT /projects/:id/zones` (bulk replace, creates new IDs)
- Zone info panel in right sidebar: label, type, area_sqft, perimeter_ft
- API types: `ApiZone` in `web/src/lib/api/types.ts`, converters for EditorZone <-> GeoJSON
- **Zone ID instability**: Bulk PUT generates new UUIDs. Tier assignments keyed by zone_id will break on zone save. Must reload assignments after zone changes.

### T-008-01: Quote API Route
- `GET /projects/{id}/quote/{tier}` in `crates/plantastic-api/src/routes/quotes.rs`
- Returns `Quote { tier, line_items, subtotal, tax, total }` with line items per zone-material pair
- Empty tier returns $0 total, no line items
- Calls `pt_quote::compute_quote()` server-side (pure computation)

## Existing Backend Endpoints (All Working)

| Endpoint | Purpose |
|---|---|
| `GET /projects/:id/zones` | List zones with area/perimeter |
| `GET /projects/:id/tiers` | Get all 3 tiers with assignments |
| `PUT /projects/:id/tiers/:tier` | Replace all assignments for one tier |
| `GET /projects/:id/quote/:tier` | Compute quote for one tier |
| `GET /materials` | List tenant material catalog |

### Tier Assignment Contract
- `PUT /projects/:id/tiers/:tier` body: `{ assignments: [{ zone_id, material_id, overrides? }] }`
- Transactional: deletes all existing, inserts new in one TX
- Constraint: `UNIQUE(project_id, tier, zone_id)` — one material per zone per tier
- Returns 204 No Content

### Quote Response Shape
```
{ tier, line_items: [{ zone_id, zone_label, material_id, material_name, quantity, unit, unit_price, line_total }], subtotal, tax, total }
```

## Frontend Architecture

### Page/Route Structure
- `/project/[id]/editor` — Zone editor (ZoneEditor.svelte + zone info panel)
- `/project/[id]/materials` — Placeholder ("Coming soon")
- `/project/[id]/quote` — Placeholder ("Coming soon")
- `/catalog` — Global material catalog CRUD (separate from project context)

### Key Components
- `ZoneEditor.svelte`: Canvas polygon drawing, mode-based (idle/drawing/selected), `selectedZoneId` state, vertex drag editing. Exposes `zones` via `$bindable`.
- `TabNav.svelte`: Project tab navigation (Editor, Materials, Quote, Viewer, Export)
- Catalog page (`catalog/+page.svelte`): Full material CRUD with modal form — pattern to reuse for material picker

### API Client Layer
- `apiFetch<T>(path, options)` in `web/src/lib/api/client.ts` — handles auth headers (Bearer + X-Tenant-Id), error classification
- `fetchZones()` / `saveZones()` in `web/src/lib/api/zones.ts` — zone-specific wrappers
- Mock API in `web/src/lib/api/mock.ts` — handles zones, materials, projects. **Missing**: tier and quote endpoints

### State Management
- `project.svelte.ts`: Store with `projects`, `current`, `zones`, `tiers` — tiers model outdated (`id`, `name`, `plantIds`), needs update
- `session.svelte.ts`: Auth token, tenant ID, active project ID
- Editor page manages its own zone state with `$state` / `$effect` — local reactive, not store-based

### Styling
- Tailwind CSS v4.2.2, brand colors: primary=#2d6a4f, secondary=#40916c, accent=#95d5b2
- Card pattern: `rounded-lg border border-gray-200 bg-white p-5`
- Category badges: hardscape=stone, softscape=green, edging=amber, fill=orange

## Key Constraints

1. **Where to build**: The `/project/[id]/materials` page is the natural home — it's a placeholder tab already in the nav. Alternatively, integrate into the editor page alongside the zone panel.
2. **Zone selection sync**: If built on the materials page, need to load zones and allow selection without the full canvas editor. If on the editor page, can reuse `selectedZoneId`.
3. **Mock API gaps**: No mock handlers for `GET /tiers`, `PUT /tiers/:tier`, or `GET /quote/:tier`. Must add for dev mode.
4. **One material per zone per tier**: DB constraint enforces this. UI must replace, not append.
5. **Zone ID drift**: After zone auto-save (bulk PUT), IDs change. Assignments reference old IDs. Need reload strategy.

## Scenario Impact
- S.3.1 (quantity computation) and S.3.2 (three-tier quote) are at OneStar. A working assignment UI enables push to ThreeStar (T-009-02's job).
- S.2.2 (material catalog search/filter) is NotImplemented — related but not this ticket's scope.
- No new scenario flips in this ticket. This is plumbing for T-009-02.

## Files That Will Be Modified/Created
- `web/src/routes/(app)/project/[id]/materials/+page.svelte` — replace placeholder
- `web/src/lib/api/mock.ts` — add tier/quote mock handlers
- `web/src/lib/api/` — new tier and quote API client modules
- New Svelte components for zone selector, material picker, tier tabs, quote summary
