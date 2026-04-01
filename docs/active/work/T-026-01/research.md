# T-026-01 Research: Loading & Error States

## Current State

### Pages with async data fetching

| Page | Route | Loading Pattern | Error Pattern | Retry? |
|------|-------|----------------|---------------|--------|
| Dashboard | `/dashboard` | 3× card skeletons (`animate-pulse`) | Red banner + retry button | Yes |
| Catalog | `/catalog` | 3× row skeletons (`animate-pulse`) | Red banner + retry button | Yes |
| Quote | `/project/[id]/quote` | Delegates to `QuoteComparison` | Inline `text-xs text-red-500` | No |
| Zone Editor | `/project/[id]/editor` | "Loading zones..." text | Inline `text-xs text-red-500` | No |
| Viewer | `/project/[id]/viewer` | "Loading viewer..." overlay (in `Viewer.svelte`) | No error handling | No |
| Materials | `/project/[id]/materials` | "Loading..." text | Inline `text-xs text-red-500` | No |

### Components with internal loading/error

| Component | File | Loading | Error |
|-----------|------|---------|-------|
| `QuoteComparison` | `lib/components/quote/QuoteComparison.svelte` | 3-column skeleton grid | None (parent shows) |
| `QuoteSummary` | `lib/components/assignment/QuoteSummary.svelte` | Has `loading` prop | None |
| `Viewer` | `lib/components/viewer/Viewer.svelte` | "Loading viewer..." overlay | `onError` callback (unused by page) |

### Data fetching pattern

All pages use the same pattern:
```ts
let loading = $state(true);
let error = $state<string | null>(null);
// ... fetch in $effect or onMount ...
.catch((e) => { error = e instanceof Error ? e.message : 'Fallback'; })
.finally(() => { loading = false; });
```

### Error classification (client.ts)

`apiFetch()` throws typed errors from `errors.ts`:
- `ApiError(status, message)` — base, for 4xx/5xx with parsed server message
- `AuthError` — 401
- `ValidationError` — 422 with field-level errors
- `RateLimitError` — 429 with retry-after header

Network errors (fetch itself fails) throw native `TypeError` — these are not caught or classified specially anywhere.

### Inline duplication

The error banner pattern appears twice (dashboard, catalog) with identical markup:
```svelte
<div class="mb-4 rounded-md bg-red-50 border border-red-200 p-4 flex items-center justify-between">
  <p class="text-sm text-red-700">{error}</p>
  <button onclick={retryFn} class="text-sm font-medium text-red-700 hover:text-red-800 underline">Retry</button>
</div>
```

The skeleton pattern appears twice with slightly different widths (w-1/3 vs w-1/4).

### What's missing

1. **No shared `LoadingSkeleton` component** — each page duplicates animate-pulse divs
2. **No shared `ErrorBanner` component** — dashboard/catalog duplicate the banner; editor/materials/quote use tiny inline text
3. **No network vs server error distinction** — all errors show `e.message` verbatim
4. **No retry on most pages** — only dashboard/catalog have retry buttons
5. **Viewer has no error display** — `onError` callback exists but viewer page doesn't use it
6. **Quote page swallows individual tier errors** — `.catch(() => null)` silently drops failures

### Scenarios affected

Scenarios with ★☆ polish that have frontend surface to improve:
- **S.2.1** (zone drawing) — editor page: text loading, inline error, no retry
- **S.2.4** (3D preview) — viewer page: loading overlay exists, no error display
- **S.3.1** (quantity computation) — quote page: skeleton via QuoteComparison, tiny inline error
- **S.3.2** (three-tier quote) — quote page: same as S.3.1

Scenarios with ★★★★★ polish (pure computation, no UX surface): S.1.1, S.1.3, S.2.2 — not affected.

### Styling tokens

- Error: `bg-red-50 border-red-200 text-red-700`
- Loading skeleton: `animate-pulse bg-gray-200/bg-gray-100 rounded`
- Brand button: `bg-brand-primary hover:bg-brand-secondary text-white rounded-md`

### Dependency: T-022-01

T-022-01 added the `Polish` enum to the scenario registry. Polish advancement from ★☆ to ★★ requires: loading indicators, error messages, empty states. This ticket delivers exactly that.
