# T-026-01 Structure: Loading & Error States

## New files

### `web/src/lib/utils/errors.ts`

Error classification utility.

```
export function friendlyError(error: unknown): string
```

Imports `ApiError` from `$lib/api/errors`. Pure function, no side effects. Classification:
- `TypeError` → network message
- `ApiError` status >= 500 → server message
- `ApiError` status 429 → rate limit message
- `ApiError` with message → pass through
- Fallback → generic message

### `web/src/lib/components/LoadingSkeleton.svelte`

Reusable loading skeleton component.

Props:
- `rows: number = 3`
- `variant: 'card' | 'row' | 'column' = 'card'`

Output: `animate-pulse` divs matching the existing skeleton patterns. No logic, purely presentational.

### `web/src/lib/components/ErrorBanner.svelte`

Reusable error banner with retry.

Props:
- `message: string`
- `onretry?: () => void`

Imports `friendlyError` from `$lib/utils/errors`. Renders the established red banner pattern. Shows Retry button only when `onretry` is provided.

## Modified files

### `web/src/routes/(app)/dashboard/+page.svelte`

- Remove inline error banner markup (lines 90-102)
- Remove inline skeleton markup (lines 104-112)
- Add `import ErrorBanner` and `import LoadingSkeleton`
- Replace with `<ErrorBanner message={error} onretry={loadProjects} />`
- Replace with `<LoadingSkeleton variant="card" />`
- Update catch block: store the error object, not just the message string

### `web/src/routes/(app)/catalog/+page.svelte`

- Same pattern as dashboard: replace inline error banner and skeleton
- Add `import ErrorBanner` and `import LoadingSkeleton`

### `web/src/routes/(app)/project/[id]/quote/+page.svelte`

- Replace inline `<span class="text-xs text-red-500">` with `<ErrorBanner>`
- Add retry function that re-fetches all three tiers
- Stop silently swallowing individual tier errors (`.catch(() => null)`)
- Catch individual tier failures but aggregate into error message

### `web/src/routes/(app)/project/[id]/editor/+page.svelte`

- Replace "Loading zones..." text with `<LoadingSkeleton variant="row" rows={4} />`
- Replace inline error span with `<ErrorBanner>` with retry
- Add a `loadZones()` function extracted from the `$effect` for retry

### `web/src/routes/(app)/project/[id]/viewer/+page.svelte`

- Add `viewerError` state variable
- Wire `onError` prop on `<Viewer>` to set `viewerError`
- Add `<ErrorBanner>` below the viewer when error occurs
- Keep existing loading overlay in Viewer.svelte (already good)

### `web/src/routes/(app)/project/[id]/materials/+page.svelte`

- Replace "Loading..." text with `<LoadingSkeleton variant="row" rows={3} />`
- Replace inline error span with `<ErrorBanner>` with retry
- Extract `loadData()` function from `$effect` for retry

## Scenario file changes

### `tests/scenarios/src/suites/design.rs`

- S.2.1: `Polish::OneStar` → `Polish::TwoStar`
- S.2.4: `Polish::OneStar` → `Polish::TwoStar`

### `tests/scenarios/src/suites/quoting.rs`

- S.3.1: `Polish::OneStar` → `Polish::TwoStar`
- S.3.2: `Polish::OneStar` → `Polish::TwoStar`

## File dependency order

1. `errors.ts` (no dependencies)
2. `LoadingSkeleton.svelte` (no dependencies)
3. `ErrorBanner.svelte` (depends on errors.ts)
4. Page updates (depend on all three above, independent of each other)
5. Scenario updates (independent of frontend changes)

## Public interfaces

### `friendlyError(error: unknown): string`
- Input: any caught error (Error, ApiError, TypeError, unknown)
- Output: human-readable string for display

### `<LoadingSkeleton>`
- `rows?: number` (default 3)
- `variant?: 'card' | 'row' | 'column'` (default 'card')

### `<ErrorBanner>`
- `message: string` (required — the raw error, will be classified internally)
- `onretry?: () => void` (optional — shows Retry button when provided)

## What is NOT changing

- `client.ts` / `apiFetch()` — error throwing behavior stays the same
- `errors.ts` — existing error classes unchanged
- `Viewer.svelte` internal loading overlay — already correct
- `QuoteComparison.svelte` internal skeleton — already correct
- Store files — no changes needed
- API mock — no changes needed
