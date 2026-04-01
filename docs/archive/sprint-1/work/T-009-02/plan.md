# Plan — T-009-02: Quote Comparison Three-Star

## Pre-implementation

1. Run `cargo run -p pt-scenarios` to capture baseline numbers.

## Step 1: Create QuoteComparison component

**File**: `web/src/lib/components/quote/QuoteComparison.svelte`

- Props: `quotes: { good: Quote | null; better: Quote | null; best: Quote | null }`, `loading: boolean`
- Derive `allZoneIds` from union of line items across all tiers
- For each zone, derive zone label (first non-null label found across tiers)
- Build lookup: `zoneItems[zoneId][tier] -> LineItem | null`
- Render: 3-column grid with tier headers, zone rows, totals
- Handle empty state: all quotes null/empty → prompt to assign materials
- Handle partial state: some tiers have assignments, some don't
- Loading state: skeleton placeholders

**Verify**: `npm run check` in web/ for type errors.

## Step 2: Wire up the quote page

**File**: `web/src/routes/(app)/project/[id]/quote/+page.svelte`

- Replace stub with data-loading page
- Fetch all 3 quotes in parallel: `Promise.all([fetchQuote(id, 'good'), fetchQuote(id, 'better'), fetchQuote(id, 'best')])`
- Handle individual tier fetch failures gracefully (set that tier's quote to null)
- Pass to QuoteComparison component
- Loading/error states

**Verify**: `npm run check` in web/ for type errors. Manual check with mock API (`VITE_MOCK_API=true`).

## Step 3: Upgrade scenario integration levels

**File**: `tests/scenarios/src/suites/quoting.rs`

- Change S.3.1 return: `OneStar` → `ThreeStar`
- Change S.3.2 return: `OneStar` → `ThreeStar`
- No assertion changes — computation correctness is unchanged

**Verify**: `cargo run -p pt-scenarios` — confirm S.3.1 and S.3.2 show ★★★☆☆ and effective minutes increase.

## Step 4: Run quality gate

- `just fmt` to auto-format
- `just lint` to check Clippy
- `just test` to run workspace tests
- `just scenarios` to verify dashboard

## Verification Criteria

1. **QuoteComparison renders 3 columns**: Good / Better / Best with totals
2. **Zone rows align**: same zone appears in same row across all tiers
3. **Monetary values formatted correctly**: `$1,530.00` format
4. **Empty tiers handled**: missing assignments show "—" not crash
5. **Loading state shown**: skeleton while fetching
6. **S.3.1 effective minutes**: 25 × 0.6 = 15 min
7. **S.3.2 effective minutes**: 15 × 0.6 = 9 min
8. **Quoting area effective**: ~24 min (up from ~8 min)
9. **No regressions**: all existing tests still pass
