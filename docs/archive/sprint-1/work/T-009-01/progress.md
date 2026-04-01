# T-009-01 Progress: Material Assignment UI

## Completed

### Step 1: API Client Modules
- Created `web/src/lib/api/tiers.ts` — `fetchTiers()`, `saveTierAssignments()` with typed interfaces
- Created `web/src/lib/api/quotes.ts` — `fetchQuote()` with `Quote` and `LineItem` types
- Both import `apiFetch` from `$lib/api` (not `./client`) to route through mock in dev mode

### Step 2: Mock API Handlers
- Added `mockTierAssignments` in-memory store keyed by `projectId:tier`
- Added `GET /projects/:id/tiers` — returns 3 tiers from mock store
- Added `PUT /projects/:id/tiers/:tier` — stores assignments in memory
- Added `GET /projects/:id/quote/:tier` — computes mock line items from zone areas + material prices

### Step 3: Assignment Components
- `TierTabs.svelte` — Three-button tab bar (Good/Better/Best) with active underline
- `ZoneList.svelte` — Clickable zone rows with type badge, area, perimeter, assigned material name
- `MaterialPicker.svelte` — Materials grouped by category, click-to-assign, checkmark on current
- `QuoteSummary.svelte` — Total, subtotal, line items with loading skeleton

### Step 4: Materials Page Orchestrator
- Replaced placeholder with full three-column layout
- Parallel data loading (zones + materials + tiers) on mount
- Optimistic assignment updates with 800ms debounced save
- Quote auto-refresh after assignment save and on tier tab switch
- Toggle unassign (click assigned material to remove)

### Step 5: Polish & Edge Cases
- Empty state for no zones ("Draw zones in the Editor tab first")
- Empty state for no materials ("No materials in catalog")
- Disabled material picker when no zone selected
- Loading skeleton in quote summary
- Error banner with save status
- Zero-dollar empty quote state

## Verification
- `svelte-check`: 0 errors, 0 warnings across 319 files
- `cargo check`: compiles clean
- `cargo test`: all tests pass, no regressions
- `cargo run -p pt-scenarios`: 25.0/240.0 min effective savings (unchanged — this ticket adds no new scenarios)

## Deviations from Plan
None. All steps executed as planned.
