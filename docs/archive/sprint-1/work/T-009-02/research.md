# Research — T-009-02: Quote Comparison Three-Star

## Objective

Build a three-tier quote comparison page (Good / Better / Best side by side) and upgrade S.3.1/S.3.2 scenarios from ★☆☆☆☆ to ★★★☆☆.

## Current State

### Scenario Baseline

- **S.3.1** (Quantity computation from geometry): PASS at OneStar — 25 min raw, 5 min effective
- **S.3.2** (Three-tier quote generation): PASS at OneStar — 15 min raw, 3 min effective
- **S.3.4** (Client quote comparison view): NotImplemented — 10 min raw, 0 min effective
- **Quoting effective total**: ~8 min of 60 min budget

### Backend — Complete

**pt-quote crate** (`crates/pt-quote/`):
- `compute_quote(zones, tier, materials, tax) -> Result<Quote, QuoteError>`
- Quote: `{ tier, line_items, subtotal, tax, total }`
- LineItem: `{ zone_id, zone_label, material_id, material_name, quantity, unit, unit_price, line_total }`
- Handles SqFt, CuYd, LinearFt, Each unit types
- 13 passing unit tests

**Quote API route** (`crates/plantastic-api/src/routes/quotes.rs`):
- `GET /projects/{id}/quote/{tier}` — loads zones, assignments, materials from DB, calls compute_quote, returns JSON
- Handles 404, 400, empty quotes

**Tier API route** (`crates/plantastic-api/src/routes/tiers.rs`):
- `GET /projects/{id}/tiers` — all 3 tiers with assignments
- `PUT /projects/{id}/tiers/{tier}` — replace assignments for a tier

### Frontend — Partial

**Existing components** (`web/src/lib/components/assignment/`):
- `TierTabs.svelte` — Good/Better/Best tab buttons with active state
- `ZoneList.svelte` — zone list with type badge, area/perimeter, assigned material
- `MaterialPicker.svelte` — material catalog grouped by category with assign action
- `QuoteSummary.svelte` — single-tier quote display (total, line items, subtotal/tax)

**Existing API layer** (`web/src/lib/api/`):
- `quotes.ts`: `fetchQuote(projectId, tier) -> Promise<Quote>` — fetches one tier
- `tiers.ts`: `fetchTiers(projectId) -> Promise<TierResponse[]>` — all 3 tiers
- `mock.ts`: full mock API with quote computation for dev without backend

**Materials page** (`web/src/routes/(app)/project/[id]/materials/+page.svelte`):
- 3-column layout: ZoneList | TierTabs+MaterialPicker | QuoteSummary
- Fetches zones, materials, tiers on mount
- Debounced save on assignment change (800ms)
- Optimistic UI updates
- Refreshes single-tier quote after save

**Quote page** (`web/src/routes/(app)/project/[id]/quote/+page.svelte`):
- **Stub**: "Coming soon." placeholder

### Navigation

`TabNav.svelte` routes: Editor | Materials | **Quote** | Viewer | Export
The Quote tab exists but links to the stub page.

### Type System

```typescript
// quotes.ts
interface Quote { tier: string; line_items: LineItem[]; subtotal: string; tax: string | null; total: string }
interface LineItem { zone_id, zone_label, material_id, material_name, quantity, unit, unit_price, line_total }

// tiers.ts
interface TierResponse { tier: 'good' | 'better' | 'best'; assignments: AssignmentResponse[] }
```

All monetary values are strings (Decimal serialization from Rust backend).

### Styling

- Tailwind CSS v4 with custom theme (`--color-brand-primary: #2d6a4f`)
- No external component library — all custom-built
- 3-column layouts used in materials page already
- `font-mono` for currency values
- Loading skeletons with `animate-pulse`

### Mock API

`mock.ts` already handles `GET /projects/:id/quote/:tier` with full computation logic (area-based quantity, unit mapping, line totals). The comparison page needs to fetch 3 quotes — mock supports this already.

## Relevant Patterns

1. **Data loading**: `$effect()` on mount/prop change → `Promise.all()` for parallel loads → set state
2. **Derived state**: `$derived.by()` for computed values
3. **Currency formatting**: `fmt(value: string)` → `$${parseFloat(value).toFixed(2)}`
4. **Unit formatting**: `formatUnit()` maps `sq_ft` → `sq ft`, etc.
5. **Layout convention**: `data: LayoutData` prop from parent layout provides `id`

## Scenario Upgrade Path

To reach ★★★ for S.3.1 and S.3.2, the scenarios need to verify that:
1. The computation is correct (already verified at ★)
2. There's a UI page that fetches and displays quote data
3. The UI shows all three tiers side by side

The scenario tests currently run pure computation. For ★★★, they should still pass their computation assertions but declare `ThreeStar` integration because a UI exists. The scenario harness doesn't test the UI directly — it trusts that if the computation is correct AND a UI exists that calls the API, the integration level can be upgraded.

## Constraints

- T-009-01 (material assignment UI) is a dependency — it's complete (all 4 components exist, materials page is wired)
- No new backend work needed — all API endpoints exist
- The mock API supports all needed endpoints
- S.3.4 (client comparison view) is a separate scenario — T-009-02 focuses on the landscaper-facing comparison
