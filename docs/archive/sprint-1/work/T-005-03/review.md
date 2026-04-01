# Review — T-005-03: Route Skeleton & API Client

## Summary of Changes

### Files Created (16)

**API Client (`web/src/lib/api/`):**
- `errors.ts` — Error class hierarchy: `ApiError` (base), `RateLimitError` (429 + retry/limit headers), `AuthError` (401), `ValidationError` (422 + field errors).
- `client.ts` — `apiFetch<T>(path, options?)` with auth header injection, JSON parsing, and typed error throwing. `sseStream(path, options)` with fetch-based SSE reader parsing `data: {json}\n\n` lines.
- `mock.ts` — Mock implementations matching the real interface. Returns hardcoded projects, materials, zones after simulated delay. Mock SSE emits 4 events.
- `index.ts` — Barrel with `VITE_MOCK_API` conditional: exports real or mock implementations. Always exports error classes.

**State Stores (`web/src/lib/stores/`):**
- `session.svelte.ts` — Svelte 5 rune-based reactive state: `tenant`, `authToken`, `activeProjectId`. Getter/setter export pattern.
- `project.svelte.ts` — Type definitions (`Project`, `Zone`, `Tier`) and reactive store: `projects`, `current`, `zones`, `tiers`, `reset()`.

**Components (`web/src/lib/components/`):**
- `Sidebar.svelte` — Navigation sidebar with brand logo, 3 nav items (Dashboard, Catalog, Settings), active state via `$page.url.pathname`, SVG icons.
- `Header.svelte` — Top bar with tenant name from session store, user avatar placeholder.
- `TabNav.svelte` — Horizontal tab bar for project workspace (Editor, Materials, Quote, Viewer, Export), active tab highlighting.

**Layouts:**
- `routes/(app)/+layout.svelte` — App shell: Sidebar + Header + scrollable main content area. Full-height flex layout.
- `routes/(app)/project/[id]/+layout.svelte` — Project workspace: heading + tab nav + child content.
- `routes/(app)/project/[id]/+layout.ts` — Passes `params.id` to layout data.

**Pages (new/updated in `(app)/`):**
- `dashboard/+page.svelte` — Project list shell with heading, "New Project" button, empty state.
- `catalog/+page.svelte` — Material list shell with heading, "Add Material" button, empty state.
- `settings/+page.svelte` — Styled heading, "Coming soon" text.
- `project/[id]/+page.svelte` — "Select a tab" prompt.
- 5 sub-route pages (editor, materials, quote, viewer, export) — placeholder content.

### Files Modified (2)

- `web/src/app.d.ts` — Defined `App.Error` interface with `message` and `status` fields.
- `web/src/lib/index.ts` — Added barrel re-exports for API client, session store, project store.

### Files Deleted (7)

Old route files moved into `(app)` group:
- `routes/dashboard/+page.svelte`
- `routes/catalog/+page.svelte`
- `routes/settings/+page.svelte`
- `routes/project/[id]/+page.svelte`, `+page.ts`
- `routes/project/[id]/{editor,materials,quote,viewer,export}/+page.svelte`

## Acceptance Criteria Check

| Criterion | Status |
|-----------|--------|
| `apiFetch` wrapper with auth headers, typed JSON, error handling | Done |
| SSE streaming reader with `data: {json}\n\n` parsing, `onPartial` callback | Done (`onEvent` callback) |
| Error types: RateLimitError (429), AuthError (401), ValidationError (422) | Done |
| Mock mode toggle (`VITE_MOCK_API`) | Done |
| App layout: sidebar (dashboard, catalog, settings) + header with tenant name | Done |
| Dashboard layout: project list shell with create button | Done |
| Project workspace: tab navigation (editor, materials, quote, viewer, export) | Done |
| Catalog layout: material list shell with add button | Done |
| All layouts render placeholder content, no API calls | Done |
| Session store: tenant, auth token, active project | Done |
| Project store: project data, zones, tiers | Done |

## Test Coverage

- **Type checking:** `svelte-check` — 306 files, 0 errors, 0 warnings.
- **Production build:** `npm run build` succeeds with CF Pages adapter.
- **Unit tests:** None added. The web project has no test framework configured (no vitest in devDependencies). API client will be exercised via mock mode when T-006-01 integrates.
- **Visual testing:** Not performed (would require `npm run dev`).

### Test Gaps

1. **No unit tests for API client.** `apiFetch` and `sseStream` have branching logic (error status codes, SSE parsing) that would benefit from unit tests. Adding vitest to web/ is a prerequisite.
2. **No unit tests for mock router.** Mock path matching is simple but untested.
3. **No visual regression.** Layout components are styled but not visually verified.

## Open Concerns

1. **`$page` import path.** Using `$app/state` (SvelteKit 2.x / Svelte 5 pattern) for `page.url.pathname` in Sidebar and TabNav. This is the correct import for Svelte 5 runes mode — `$app/stores` would use the legacy `$page` store.

2. **SSE stream error handling.** If the fetch-based SSE reader encounters a non-JSON `data:` line, it catches the parse error and calls `onError`. The stream continues reading. This is correct but callers should be aware.

3. **Mock data is minimal.** Enough for development scaffolding but will need expansion as T-006-01 builds out real UI interactions.

4. **No mobile responsiveness.** Sidebar is fixed-width (14rem). No hamburger menu or collapse behavior on small screens. This is acceptable for the current stage but will need attention before client-facing use.

5. **Landing page (`/`) and client view (`/c/[token]`) bypass the app shell.** This is intentional — they sit outside the `(app)` route group. Future tickets may add their own layouts.

6. **Auth flow is stubbed.** The session store holds an `authToken` but nothing populates it. `apiFetch` sends an empty `Authorization` header when no token is set — the header is omitted entirely (correct behavior).
