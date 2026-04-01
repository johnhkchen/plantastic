# T-026-01 Progress: Loading & Error States

## Completed

### Step 1: `friendlyError` utility ✓
- Created `web/src/lib/utils/errors.ts`
- Classifies TypeError (network), ApiError 5xx (server), 429 (rate limit), and fallback

### Step 2: `LoadingSkeleton.svelte` ✓
- Created `web/src/lib/components/LoadingSkeleton.svelte`
- Three variants: card, row, column
- Configurable row count

### Step 3: `ErrorBanner.svelte` ✓
- Created `web/src/lib/components/ErrorBanner.svelte`
- Calls `friendlyError()` internally for human-readable messages
- Optional retry button

### Step 4: Dashboard page ✓
- Replaced inline error banner and skeleton with shared components
- Existing retry via `loadProjects` preserved

### Step 5: Catalog page ✓
- Replaced inline error banner and skeleton with shared components
- Existing retry via `loadMaterials` preserved

### Step 6: Quote page ✓
- Extracted `loadQuotes()` function for retry
- Replaced inline error span with `ErrorBanner` with retry
- Added detection when all three tiers fail

### Step 7: Editor page ✓
- Extracted `loadZones()` function for retry
- Replaced bare "Loading zones..." text with `LoadingSkeleton variant="row"`
- Replaced inline error span with `ErrorBanner` with retry

### Step 8: Viewer page ✓
- Added `viewerError` state
- Wired `onError` callback to display `ErrorBanner`

### Step 9: Materials page ✓
- Extracted `loadData()` function for retry
- Replaced bare "Loading..." text with `LoadingSkeleton variant="row"`
- Replaced inline error span with `ErrorBanner` with retry

### Step 10: Scenario polish ratings ✓
- S.2.1: Polish::OneStar → Polish::TwoStar
- S.2.4: Polish::OneStar → Polish::TwoStar
- S.3.1: Polish::OneStar → Polish::TwoStar (4 occurrences: API + computation paths)
- S.3.2: Polish::OneStar → Polish::TwoStar (4 occurrences: API + computation paths)

## Verification

- `just check` passes (format, lint, test, scenarios)
- Effective savings: 69.5 → 76.5 min (+7.0 min, +2.9 percentage points)
- Polish debt: 40.0 → 33.0 min (-7.0 min recovered)
- No regressions: 9 pass, 0 fail (unchanged)

## Deviations from plan

None. All steps executed as planned.
