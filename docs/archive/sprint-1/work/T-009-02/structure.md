# Structure — T-009-02: Quote Comparison Three-Star

## Files Modified

### 1. `web/src/routes/(app)/project/[id]/quote/+page.svelte` — REWRITE

Replace "Coming soon" stub with the comparison page.

**Responsibilities:**
- Fetch all 3 quotes in parallel on mount/project change
- Pass quotes to QuoteComparison component
- Handle loading/error states

**Props from layout**: `data: LayoutData` (provides `data.id`)

**State:**
- `quotes: Record<string, Quote | null>` — keyed by tier name
- `loading: boolean`
- `error: string | null`

**Data loading pattern**: Same `$effect()` + `Promise.all()` as materials page.

### 2. `web/src/lib/components/quote/QuoteComparison.svelte` — NEW

**Props:**
```typescript
{
  quotes: { good: Quote | null; better: Quote | null; best: Quote | null };
  loading: boolean;
}
```

**Internal structure:**
- `allZones` — derived: union of zone_ids across all three quotes, preserving order
- For each zone, look up the line item in each tier (or null if not assigned)

**Layout (3-column grid):**
```
┌────────────────────────────────────────────────────────────┐
│  GOOD              │  BETTER            │  BEST            │
│  $839.26           │  $1,748.89         │  $3,148.40       │
├────────────────────┼────────────────────┼────────────────────┤
│  Back patio        │  Back patio        │  Back patio        │
│  Concrete Pavers   │  Travertine Pavers │  Bluestone Pavers  │
│  180 sq ft @ $4.00 │  180 sq ft @ $8.50 │  180 sq ft @ $15   │
│  $720.00           │  $1,530.00         │  $2,700.00         │
├────────────────────┼────────────────────┼────────────────────┤
│  Garden bed        │  Garden bed        │  Garden bed        │
│  Basic Mulch       │  Premium Mulch     │  Cedar Mulch       │
│  2.0 cu yd @ $30   │  2.0 cu yd @ $45   │  2.0 cu yd @ $65   │
│  $59.26            │  $88.89            │  $128.40           │
├────────────────────┼────────────────────┼────────────────────┤
│  Border edging     │  Border edging     │  Border edging     │
│  Plastic Edge      │  Steel Edge        │  Corten Edge       │
│  40 lin ft @ $1.50 │  40 lin ft @ $3.25 │  40 lin ft @ $8    │
│  $60.00            │  $130.00           │  $320.00           │
└────────────────────┴────────────────────┴────────────────────┘
```

**Zone alignment logic:**
```typescript
// Collect unique zone_ids from all tiers, preserving first-seen order
const allZoneIds: string[] = [];
const seen = new Set<string>();
for (const tier of ['good', 'better', 'best']) {
  for (const li of quotes[tier]?.line_items ?? []) {
    if (!seen.has(li.zone_id)) { seen.add(li.zone_id); allZoneIds.push(li.zone_id); }
  }
}
```

For each zone_id × tier: find the matching line item or show "—".

**Visual emphasis:**
- Tier headers: larger font, bold total
- Column colors: Good = gray/muted, Better = brand-primary (recommended), Best = premium/darker
- Price differences: no explicit diff calculation needed — alignment makes comparison visual

### 3. `tests/scenarios/src/suites/quoting.rs` — MODIFY

**Change 1** (line 218): `s_3_1_quantity_from_geometry` return value
```rust
// Before:
ScenarioOutcome::Pass(Integration::OneStar)
// After:
ScenarioOutcome::Pass(Integration::ThreeStar)
```

**Change 2** (line 503): `s_3_2_three_tier_quotes` return value
```rust
// Before:
ScenarioOutcome::Pass(Integration::OneStar)
// After:
ScenarioOutcome::Pass(Integration::ThreeStar)
```

## Files NOT Modified

- Backend crates: no changes (all API endpoints exist)
- `QuoteSummary.svelte`: stays as-is for the materials page sidebar
- `TierTabs.svelte`: not used on comparison page (all 3 tiers shown at once)
- `mock.ts`: already handles all needed API calls
- `quotes.ts` API: `fetchQuote` already takes a tier param
- `TabNav.svelte`: Quote tab already links to the right route

## Component Hierarchy

```
quote/+page.svelte
└── QuoteComparison.svelte (new)
```

## Directory for New Component

`web/src/lib/components/quote/` — new directory, separates comparison from assignment components.
