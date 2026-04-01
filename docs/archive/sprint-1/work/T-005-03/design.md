# Design — T-005-03: Route Skeleton & API Client

## Decision 1: API Client Architecture

### Option A: Single `apiFetch` function + SSE helper (chosen)

A `client.ts` module exporting:
- `apiFetch<T>(path, options?)` — wraps `fetch`, injects auth, parses JSON, throws typed errors.
- `sseStream(path, options?, onEvent)` — manual SSE via `fetch` + `ReadableStream` reader (not `EventSource`), calls `onEvent` per parsed `data:` line.
- Custom error classes: `ApiError` (base), `RateLimitError`, `AuthError`, `ValidationError`.
- A `mock.ts` module with the same interface, swapped in when `VITE_MOCK_API` is truthy.

**Why chosen:** Matches HMW Workshop pattern. `fetch`-based SSE gives control over headers (auth token) that `EventSource` cannot set. Error classes enable `instanceof` pattern matching in callers. Mock module keeps the interface identical — callers don't know they're in mock mode.

### Option B: Class-based API client (rejected)

An `ApiClient` class instantiated with config (baseUrl, token). Methods like `client.get<T>(path)`, `client.post<T>(path, body)`, `client.stream(path, onEvent)`.

**Why rejected:** Adds OOP indirection for no benefit. The client is stateless — auth token comes from the session store at call time. A class instance would need to be a singleton or passed through context, which is heavier than importing a function. The HMW pattern proves the function approach works.

### Option C: SvelteKit `fetch` passthrough (rejected)

Use SvelteKit's `event.fetch` in `+page.server.ts` load functions to make API calls server-side, then pass data to pages.

**Why rejected:** The app deploys to CF Pages (edge runtime). Server-side calls from the edge to Lambda add latency vs. direct client-to-worker calls. SSE streaming doesn't fit server load functions. The worker proxy already handles CORS, so client-side fetch is the right pattern.

## Decision 2: Mock Mode Implementation

### Option A: Module-level conditional export (chosen)

```ts
// api/index.ts
const useMock = import.meta.env.VITE_MOCK_API === 'true';
export const apiFetch = useMock ? mockApiFetch : realApiFetch;
export const sseStream = useMock ? mockSseStream : realSseStream;
```

Mock module returns hardcoded responses after a small delay (simulating network). Re-export through a single `api/index.ts` barrel.

**Why chosen:** Dead-code eliminated by Vite in production builds. Callers import from `$lib/api` and never think about mocking. Easy to add new mock endpoints.

### Option B: Runtime flag checked inside each function (rejected)

**Why rejected:** Pollutes real code with mock branches. Not tree-shakeable.

## Decision 3: Layout Architecture

### Option A: Route group layouts (chosen)

```
routes/
├── +layout.svelte              # Bare shell (CSS, favicon) — unchanged
├── +page.svelte                # Landing — no app shell
├── (app)/
│   ├── +layout.svelte          # App shell: sidebar + header + main area
│   ├── dashboard/+page.svelte
│   ├── catalog/+page.svelte
│   ├── settings/+page.svelte
│   └── project/[id]/
│       ├── +layout.svelte      # Tab navigation bar
│       └── ...sub-routes
├── c/[token]/+page.svelte      # Client view — no app shell
```

SvelteKit route groups `(app)` apply the sidebar layout to dashboard/catalog/settings/project without affecting the landing page or client view.

**Why chosen:** Clean separation. Landing and client views don't need `if` checks to hide sidebar. The group name `(app)` doesn't appear in the URL. SvelteKit handles layout nesting automatically.

### Option B: Single root layout with conditional rendering (rejected)

Check `$page.url.pathname` in root layout to conditionally show/hide sidebar.

**Why rejected:** Fragile. Every new non-sidebar route requires updating the condition. Route groups solve this structurally.

### Option C: Separate layout per section (rejected)

Dashboard, catalog, settings each get their own full layout with duplicated sidebar.

**Why rejected:** Duplicates sidebar/header across layouts. Changes require updating multiple files.

## Decision 4: State Management

### Option A: Svelte 5 rune modules (chosen)

Plain `.svelte.ts` files exporting reactive state:

```ts
// stores/session.svelte.ts
let tenant = $state('');
let authToken = $state('');
let activeProjectId = $state<string | null>(null);

export const session = {
  get tenant() { return tenant; },
  set tenant(v) { tenant = v; },
  // ...
};
```

**Why chosen:** Idiomatic Svelte 5. No writable stores, no context API boilerplate. Importable from anywhere. Getter/setter pattern keeps reactivity intact when destructured in components.

### Option B: Svelte context API (rejected)

Use `setContext`/`getContext` in root layout.

**Why rejected:** Requires component tree dependency. Can't access stores from `$lib/api` module (which runs outside component context). Rune modules are universally importable.

### Option C: Legacy `writable()` stores (rejected)

**Why rejected:** Svelte 5 runes are the modern pattern. The project already uses runes throughout. Mixing paradigms adds confusion.

## Decision 5: Component Granularity

### Approach: Minimal shared components

Create only what's needed for the layouts in this ticket:
- `Sidebar.svelte` — Navigation links (dashboard, catalog, settings), brand logo.
- `Header.svelte` — Tenant name placeholder, user avatar placeholder.
- `TabNav.svelte` — Horizontal tab bar for project workspace sub-routes.

No component library, no generic Button/Card/etc. Pages use Tailwind utilities directly. Components are extracted only when the same markup appears in multiple layouts.

**Rationale:** YAGNI. T-006-01 and later tickets will introduce real UI complexity. Premature component extraction now would need to be reworked when actual data flows are known.

## Summary of Decisions

| # | Decision | Choice |
|---|----------|--------|
| 1 | API client | Function-based: `apiFetch<T>` + `sseStream` + error classes |
| 2 | Mock mode | Module-level conditional export, tree-shakeable |
| 3 | Layouts | Route group `(app)` for sidebar shell, nested layout for project tabs |
| 4 | State | Svelte 5 `.svelte.ts` rune modules with getter/setter exports |
| 5 | Components | Sidebar, Header, TabNav — minimal set for layout needs |
