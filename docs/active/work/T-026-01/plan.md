# T-026-01 Plan: Loading & Error States

## Step 1: Create `friendlyError` utility

**File:** `web/src/lib/utils/errors.ts`

Create the error classification function. Import `ApiError` from `$lib/api/errors`. Implement the classification logic (TypeError → network, 5xx → server, 429 → rate limit, else → pass through or fallback).

**Verify:** TypeScript compiles. Function handles all error types including `unknown`.

## Step 2: Create `LoadingSkeleton.svelte`

**File:** `web/src/lib/components/LoadingSkeleton.svelte`

Create the component with `rows` and `variant` props. Implement three variants:
- `card`: the dashboard/catalog style (rounded-lg border card with two gray bars)
- `row`: the editor/materials style (simpler horizontal bars)
- `column`: the quote comparison style (column with header + body placeholders)

Extract the exact markup from existing pages to ensure visual consistency.

**Verify:** Component renders correctly in each variant.

## Step 3: Create `ErrorBanner.svelte`

**File:** `web/src/lib/components/ErrorBanner.svelte`

Create the component with `message` and optional `onretry` props. Calls `friendlyError(message)` to get user-visible text. Renders the red banner. Shows Retry button conditionally.

**Verify:** Component renders with and without retry button.

## Step 4: Update dashboard page

**File:** `web/src/routes/(app)/dashboard/+page.svelte`

- Import `ErrorBanner` and `LoadingSkeleton`
- Replace inline error banner with `<ErrorBanner message={error} onretry={loadProjects} />`
- Replace inline skeleton with `<LoadingSkeleton variant="card" />`
- Keep catch block storing error as string (ErrorBanner handles classification via friendlyError)

**Verify:** Page shows skeleton on load, error banner on failure, retry works.

## Step 5: Update catalog page

**File:** `web/src/routes/(app)/catalog/+page.svelte`

Same pattern as dashboard: replace inline markup with shared components.

**Verify:** Skeleton on load, error banner on failure, retry works.

## Step 6: Update quote page

**File:** `web/src/routes/(app)/project/[id]/quote/+page.svelte`

- Import `ErrorBanner`
- Add a `loadQuotes()` function extracted from the `$effect` body
- Replace the inline error span with `<ErrorBanner>` with retry
- Keep the `.catch(() => null)` on individual tiers but track if ALL three fail

**Verify:** Error banner shows on failure, retry re-fetches all tiers.

## Step 7: Update editor page

**File:** `web/src/routes/(app)/project/[id]/editor/+page.svelte`

- Import `ErrorBanner` and `LoadingSkeleton`
- Extract `loadZones()` function from `$effect` for retry
- Replace "Loading zones..." text with `<LoadingSkeleton variant="row" rows={4} />`
- Replace inline error span with `<ErrorBanner>` with retry

**Verify:** Skeleton overlay during load, error banner on failure, retry works.

## Step 8: Update viewer page

**File:** `web/src/routes/(app)/project/[id]/viewer/+page.svelte`

- Import `ErrorBanner`
- Add `viewerError` state
- Wire `onError` to set `viewerError`
- Show `<ErrorBanner>` when error occurs

**Verify:** Error banner appears when viewer reports error.

## Step 9: Update materials page

**File:** `web/src/routes/(app)/project/[id]/materials/+page.svelte`

- Import `ErrorBanner` and `LoadingSkeleton`
- Extract `loadData()` function from `$effect` for retry
- Replace "Loading..." text with `<LoadingSkeleton variant="row" />`
- Replace inline error span with `<ErrorBanner>` with retry

**Verify:** Skeleton during load, error banner on failure, retry works.

## Step 10: Update scenario polish ratings

**Files:**
- `tests/scenarios/src/suites/design.rs`: S.2.1 and S.2.4 → `Polish::TwoStar`
- `tests/scenarios/src/suites/quoting.rs`: S.3.1 and S.3.2 → `Polish::TwoStar`

Update the comments to document why the polish advanced.

**Verify:** `cargo run -p pt-scenarios` shows increased effective minutes. `just check` passes.

## Testing strategy

This ticket is primarily frontend UI work. The verification is:
1. **Scenario dashboard** — effective minutes increase by ~7.0 (from polish advancement)
2. **`just check`** — format, lint, test, scenarios all pass
3. **Visual inspection** — loading skeletons and error banners render correctly (manual)

No new Rust unit tests needed — the changes are Svelte components and scenario rating updates. The scenario harness itself is the test.
