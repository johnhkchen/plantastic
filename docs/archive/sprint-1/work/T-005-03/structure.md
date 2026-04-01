# Structure — T-005-03: Route Skeleton & API Client

## File Map

### New Files

```
web/src/lib/
├── api/
│   ├── client.ts           # apiFetch<T>, sseStream, base URL resolution
│   ├── errors.ts           # ApiError, RateLimitError, AuthError, ValidationError
│   ├── mock.ts             # Mock implementations of apiFetch and sseStream
│   └── index.ts            # Barrel: conditional export (real vs mock based on VITE_MOCK_API)
├── stores/
│   ├── session.svelte.ts   # tenant, authToken, activeProjectId
│   └── project.svelte.ts   # project data, zones, tiers (empty shell — populated by T-006-01)
└── components/
    ├── Sidebar.svelte      # Nav links: dashboard, catalog, settings; brand logo
    ├── Header.svelte       # Tenant name placeholder, user avatar placeholder
    └── TabNav.svelte       # Horizontal tabs for project workspace sub-routes
```

### Route Restructure

Move dashboard, catalog, settings, and project routes into an `(app)` route group with a shared layout.

**Before:**
```
web/src/routes/
├── +layout.svelte
├── +page.svelte
├── dashboard/+page.svelte
├── catalog/+page.svelte
├── settings/+page.svelte
├── project/[id]/...
└── c/[token]/+page.svelte
```

**After:**
```
web/src/routes/
├── +layout.svelte                          # Root: CSS + favicon (unchanged)
├── +page.svelte                            # Landing (unchanged)
├── (app)/
│   ├── +layout.svelte                      # NEW: App shell (Sidebar + Header + main)
│   ├── dashboard/
│   │   └── +page.svelte                    # MODIFIED: project list shell + create button
│   ├── catalog/
│   │   └── +page.svelte                    # MODIFIED: material list shell + add button
│   ├── settings/
│   │   └── +page.svelte                    # MOVED: unchanged content
│   └── project/[id]/
│       ├── +layout.svelte                  # NEW: Tab navigation bar
│       ├── +page.svelte                    # MODIFIED: project overview / redirect
│       ├── +page.ts                        # MOVED: unchanged
│       ├── editor/+page.svelte             # MOVED: unchanged
│       ├── materials/+page.svelte          # MOVED: unchanged
│       ├── quote/+page.svelte              # MOVED: unchanged
│       ├── viewer/+page.svelte             # MOVED: unchanged
│       └── export/+page.svelte             # MOVED: unchanged
└── c/[token]/+page.svelte                  # Client view (unchanged, outside app group)
```

### Modified Files

- `web/src/app.d.ts` — Add `App.Error` interface matching API error shape.

### Deleted Files

After moving into `(app)/`:
- `web/src/routes/dashboard/+page.svelte` (old location)
- `web/src/routes/catalog/+page.svelte` (old location)
- `web/src/routes/settings/+page.svelte` (old location)
- `web/src/routes/project/` (old directory)

## Module Boundaries

### `$lib/api` — API Client

**Public interface (`index.ts`):**
```ts
export { apiFetch } from './client' | './mock';   // conditional
export { sseStream } from './client' | './mock';   // conditional
export { ApiError, RateLimitError, AuthError, ValidationError } from './errors';
export type { ApiOptions, SseOptions } from './client';
```

**`client.ts` exports:**
- `apiFetch<T>(path: string, options?: ApiOptions): Promise<T>` — GET/POST/PUT/DELETE with auth header injection, JSON parsing, typed error throwing.
- `sseStream(path: string, options: SseOptions): Promise<void>` — Fetch-based SSE reader. Reads `data: {json}\n\n` lines from a streaming response. Calls `onEvent(parsed)` for each event. Resolves when stream ends. Rejects on error.

**`errors.ts` exports:**
- `ApiError` — base class. Fields: `status: number`, `message: string`.
- `RateLimitError extends ApiError` — Fields: `retryAfter: number` (seconds), `limit: number`, `remaining: number`.
- `AuthError extends ApiError` — 401 responses.
- `ValidationError extends ApiError` — 422 responses. Fields: `errors: Record<string, string[]>`.

**`mock.ts` exports:**
- `mockApiFetch<T>(path: string, options?: ApiOptions): Promise<T>` — Returns hardcoded JSON after 100-300ms delay. Path-based routing to mock data.
- `mockSseStream(path: string, options: SseOptions): Promise<void>` — Emits 3-5 mock events with delays, then resolves.

### `$lib/stores` — State Management

**`session.svelte.ts` exports:**
```ts
export const session: {
  tenant: string;         // Current tenant name (placeholder)
  authToken: string;      // Bearer token (empty until login)
  activeProjectId: string | null;
};
```

**`project.svelte.ts` exports:**
```ts
export const projectStore: {
  projects: Project[];    // List for dashboard
  current: Project | null; // Active project in workspace
  zones: Zone[];          // Zones for current project
  tiers: Tier[];          // Tiers for current project
};

// Type definitions
export interface Project { id: string; name: string; createdAt: string; }
export interface Zone { id: string; name: string; area: number; }
export interface Tier { id: string; name: string; plantIds: string[]; }
```

### `$lib/components` — Shared UI

**`Sidebar.svelte`:**
- Props: none (reads current path from `$page`).
- Renders: brand logo, nav links with active state highlighting.
- Nav items: Dashboard (`/dashboard`), Catalog (`/catalog`), Settings (`/settings`).

**`Header.svelte`:**
- Props: none (reads `session.tenant` from store).
- Renders: tenant name, user avatar placeholder, horizontal bar at top.

**`TabNav.svelte`:**
- Props: `projectId: string` (for building tab hrefs).
- Renders: horizontal tab row. Tabs: Editor, Materials, Quote, Viewer, Export.
- Active tab highlighted based on `$page.url.pathname`.

## Ordering of Changes

1. **Create `$lib/api/errors.ts`** — no dependencies, needed by everything else.
2. **Create `$lib/api/client.ts`** — depends on errors.ts and session store (for auth token).
3. **Create `$lib/stores/session.svelte.ts`** — no dependencies.
4. **Create `$lib/stores/project.svelte.ts`** — no dependencies.
5. **Create `$lib/api/mock.ts`** — depends on errors.ts, mirrors client.ts interface.
6. **Create `$lib/api/index.ts`** — barrel that conditionally re-exports.
7. **Create components:** Sidebar, Header, TabNav.
8. **Restructure routes:** Create `(app)/` group, move routes, add layouts.
9. **Update `app.d.ts`** — Add error interface.
