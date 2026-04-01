# T-009-01 Structure: Material Assignment UI

## Files Modified

### `web/src/routes/(app)/project/[id]/materials/+page.svelte`
Replace placeholder with the material assignment page. This is the main orchestrator:
- Loads zones, materials, and tier assignments on mount
- Manages `activeTier`, `selectedZoneId`, `quote` state
- Coordinates assignment saves and quote refreshes
- Three-column layout: ZoneList | MaterialPicker (with tier tabs) | QuoteSummary

### `web/src/lib/api/mock.ts`
Add mock handlers for:
- `GET /projects/:id/tiers` — return 3 tiers from in-memory store
- `PUT /projects/:id/tiers/:tier` — store assignments, return 204
- `GET /projects/:id/quote/:tier` — compute mock quote from assignments + materials + zones

## Files Created

### `web/src/lib/api/tiers.ts`
API client for tier endpoints:
- `fetchTiers(projectId): Promise<TierResponse[]>` — GET all 3 tiers
- `saveTierAssignments(projectId, tier, assignments): Promise<void>` — PUT assignments

Types:
```ts
interface TierResponse { tier: string; assignments: AssignmentResponse[] }
interface AssignmentResponse { zone_id: string; material_id: string; overrides: unknown | null }
interface AssignmentInput { zone_id: string; material_id: string; overrides?: unknown }
```

### `web/src/lib/api/quotes.ts`
API client for quote endpoint:
- `fetchQuote(projectId, tier): Promise<Quote>` — GET computed quote

Types:
```ts
interface Quote { tier: string; line_items: LineItem[]; subtotal: string; tax: string | null; total: string }
interface LineItem { zone_id: string; zone_label: string | null; material_id: string; material_name: string; quantity: string; unit: string; unit_price: string; line_total: string }
```

### `web/src/lib/components/assignment/ZoneList.svelte`
Left panel component. Props:
- `zones: ApiZone[]`
- `selectedZoneId: string | null` (bindable)
- `assignments: AssignmentResponse[]` (for current tier)
- `materials: Material[]` (to show assigned material name)

Renders each zone as a clickable row with type badge, area, and current material name.

### `web/src/lib/components/assignment/MaterialPicker.svelte`
Center panel component. Props:
- `materials: Material[]`
- `selectedZoneId: string | null`
- `currentMaterialId: string | null` (material assigned to selected zone in active tier)
- `onAssign: (materialId: string) => void` callback

Groups materials by category. Each material row is clickable. Currently assigned material has a checkmark. Disabled state when no zone is selected.

### `web/src/lib/components/assignment/TierTabs.svelte`
Tab bar for good/better/best. Props:
- `activeTier: string` (bindable)

Three styled tabs. Active tab gets underline + bold.

### `web/src/lib/components/assignment/QuoteSummary.svelte`
Right panel component. Props:
- `quote: Quote | null`
- `loading: boolean`

Shows total, subtotal, and line items. Loading skeleton when computing.

## Module Boundaries

```
materials/+page.svelte (orchestrator)
├── imports ZoneList, MaterialPicker, TierTabs, QuoteSummary
├── imports fetchZones (existing)
├── imports fetchTiers, saveTierAssignments (new)
├── imports fetchQuote (new)
├── imports apiFetch for materials (existing pattern)
└── manages all state, passes down via props + callbacks

Components are pure display + callbacks — no API calls, no side effects.
```

## Files NOT Modified
- `ZoneEditor.svelte` — no changes needed. Zone selection is handled by ZoneList, not canvas.
- `editor/+page.svelte` — unchanged. Editor tab and Materials tab are independent.
- `TabNav.svelte` — already has Materials tab link.
- Backend Rust code — all endpoints already exist.
- `project.svelte.ts` — global store not used. Local state in page component.

## Ordering
1. API clients first (tiers.ts, quotes.ts) — no dependencies
2. Mock API handlers — needed for dev testing
3. Components (ZoneList, MaterialPicker, TierTabs, QuoteSummary) — can be built in parallel
4. Page orchestrator last — wires everything together
