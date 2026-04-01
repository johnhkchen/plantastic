# T-026-01 Design: Loading & Error States

## Decision: Extract shared components, add error classification

### Approach A: Shared Svelte components (chosen)

Create two reusable components:
1. `LoadingSkeleton.svelte` — configurable skeleton placeholder
2. `ErrorBanner.svelte` — error banner with retry, human-readable messages

Add a utility function `friendlyError()` to classify errors as network vs server vs client.

Replace inline loading/error markup across all 6 pages with these components.

**Why chosen:** Matches the ticket's acceptance criteria ("shared `LoadingState` pattern, not ad-hoc per page"). Minimal API surface — two components and one function. No new abstractions, stores, or context providers needed. Each page still manages its own `loading`/`error` state; the components are purely presentational.

### Approach B: Global error context + toast system

Wrap the app layout in an error context provider. All `apiFetch` calls automatically push errors to a global toast queue. Loading states managed via a shared `LoadingState` store.

**Why rejected:** Over-engineered for current needs. Toast system adds complexity (queue, animation, auto-dismiss). Global state coupling makes it harder to have page-specific retry actions. The ticket asks for "simple spinners/skeletons, not shimmer animations."

### Approach C: SvelteKit `{#await}` blocks

Use SvelteKit's native `{#await promise}` template blocks instead of manual loading/error state.

**Why rejected:** Most pages need to interact with loaded data after fetch (e.g., assign materials, draw zones). `{#await}` is good for fire-and-forget display but awkward when the data needs to be mutable state. The existing `$effect` + `$state` pattern is already established and works well.

## Component Design

### `ErrorBanner.svelte`

```svelte
Props:
  message: string        — raw error message (from catch block)
  onretry?: () => void   — retry callback; if provided, shows Retry button
```

Internally calls `friendlyError(message)` to produce human-readable text. Distinguishes:
- Network errors ("Couldn't reach the server. Check your connection and try again.")
- Server errors / 5xx ("Something went wrong. Please try again in a moment.")
- Client errors / specific messages (pass through the server message)

Uses the established red styling tokens: `bg-red-50 border-red-200 text-red-700`.

### `LoadingSkeleton.svelte`

```svelte
Props:
  rows?: number          — number of skeleton rows (default 3)
  variant?: 'card' | 'row' | 'column'  — skeleton shape
```

Variants:
- `card` — full card skeleton (dashboard project list, catalog material list)
- `row` — inline row skeleton (single line items)
- `column` — tier column skeleton (quote comparison)

Uses `animate-pulse` with gray backgrounds. No shimmer, no fancy animations.

### `friendlyError()` utility

```ts
function friendlyError(error: unknown): string
```

Classification logic:
1. If error is `TypeError` with "fetch" or "Failed to fetch" → network message
2. If error is `ApiError` with status >= 500 → server message
3. If error is `ApiError` with status 429 → rate limit message
4. Otherwise → use error.message with a friendly prefix

Located in `$lib/utils/errors.ts`.

## Pages to update

| Page | Loading change | Error change |
|------|---------------|-------------|
| Dashboard | Replace inline skeleton → `<LoadingSkeleton variant="card" />` | Already has banner — replace with `<ErrorBanner>` |
| Catalog | Replace inline skeleton → `<LoadingSkeleton variant="card" />` | Already has banner — replace with `<ErrorBanner>` |
| Quote | Already delegates to QuoteComparison | Replace inline span → `<ErrorBanner>` with retry |
| Editor | Replace "Loading zones..." text → `<LoadingSkeleton variant="row" />` overlay | Replace inline span → `<ErrorBanner>` with retry |
| Viewer | Keep existing overlay (already good) | Wire `onError` → `<ErrorBanner>` |
| Materials | Replace "Loading..." text → `<LoadingSkeleton variant="row" />` | Replace inline span → `<ErrorBanner>` with retry |

## Error message classification

| Condition | User sees |
|-----------|-----------|
| `TypeError` / network failure | "Couldn't reach the server. Check your connection and try again." |
| `ApiError` status >= 500 | "Something went wrong on our end. Please try again." |
| `ApiError` status 429 | "Too many requests. Please wait a moment and try again." |
| `ApiError` with specific message | The server's message (already human-readable from our API) |
| Unknown error | "An unexpected error occurred. Please try again." |

## Scenario advancement

After this work, scenarios S.2.1, S.2.4, S.3.1, S.3.2 advance from `Polish::OneStar` to `Polish::TwoStar`. This represents adding loading indicators and error handling to their UX surfaces — exactly the ★☆→★★ advancement criteria from the testing strategy.

Effective minutes impact:
- S.2.1: 20 × (2+2)/10 = 8.0 (was 20 × (2+1)/10 = 6.0, +2.0)
- S.2.4: 10 × (2+2)/10 = 4.0 (was 10 × (2+1)/10 = 3.0, +1.0)
- S.3.1: 25 × (3+2)/10 = 12.5 (was 25 × (3+1)/10 = 10.0, +2.5)
- S.3.2: 15 × (3+2)/10 = 7.5 (was 15 × (3+1)/10 = 6.0, +1.5)
- Total: +7.0 effective minutes
