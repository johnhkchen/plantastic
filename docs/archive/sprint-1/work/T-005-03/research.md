# Research — T-005-03: Route Skeleton & API Client

## Current State

### Web Scaffold (T-005-01 — done)

SvelteKit 5 app in `web/` with Cloudflare Pages adapter, Tailwind CSS 4, TypeScript strict mode.

**Route tree** (all placeholder "Coming soon"):
```
src/routes/
├── +layout.svelte          # Root: imports app.css + favicon, renders children
├── +page.svelte            # Landing: "Plantastic" heading
├── dashboard/+page.svelte
├── catalog/+page.svelte
├── settings/+page.svelte
├── project/[id]/
│   ├── +page.svelte        # Shows "Project {data.id}"
│   ├── +page.ts            # load({ params }) → { id: params.id }
│   ├── editor/+page.svelte
│   ├── materials/+page.svelte
│   ├── quote/+page.svelte
│   ├── viewer/+page.svelte
│   └── export/+page.svelte
└── c/[token]/+page.svelte  # Client-facing branded view
```

**Key observations:**
- Root `+layout.svelte` is bare — just CSS import and favicon. No navigation, no sidebar, no header.
- No nested layouts exist (`project/[id]/+layout.svelte` is missing).
- `src/lib/` is empty (only `index.ts` placeholder and `assets/favicon.svg`).
- No stores, no components, no API module.
- Svelte 5 runes syntax in use: `$props()`, `{@render children()}`.
- No `+layout.ts` files (no server-side data loading).

**Styling:**
- Tailwind CSS 4 via `@tailwindcss/vite` plugin.
- Brand theme in `app.css`: `--color-brand-primary: #2d6a4f`, `--color-brand-secondary: #40916c`, `--color-brand-accent: #95d5b2`.
- Fonts: Inter (display + body).

**Build config:**
- Vite dev proxy: `/api` → `process.env.API_URL || 'http://localhost:3000'`.
- Cloudflare Pages adapter (no special config).
- No dependencies beyond SvelteKit, Tailwind, TypeScript, ESLint, Prettier.

### CF Worker Proxy (T-005-02 — done)

Worker in `worker/` proxies `/api/*` and `/health` to Lambda backend.

**Relevant to API client design:**
- Routes: `/api/*` (GET/POST only), `/health` (GET).
- Auth: `Authorization` header forwarded verbatim.
- Rate limit headers returned: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`.
- Rate limit exceeded → 429 with `Retry-After` header and JSON `{ error: "..." }`.
- SSE: response body streamed without buffering (no special content-type handling).
- CORS: handled at worker level; frontend doesn't need to worry about CORS in production.
- Error responses are always JSON: `{ error: string }`.

### Backend API (not yet built)

No Axum Lambda backend exists yet (T-004-01 and T-004-02 are not done). The API client must work in mock mode until the backend is ready. The ticket explicitly calls for `VITE_MOCK_API` env var toggle.

### HMW Workshop Pattern (reference)

From memory: the HMW Workshop used a similar stack (SvelteKit + CF Worker + Lambda). Key patterns:
- `apiFetch` wrapper with auth headers and typed responses.
- SSE streaming reader for `data: {json}\n\n` events.
- Mock mode for dev without backend.
- Frontend-owned state, stateless backend.

## Constraints & Boundaries

1. **No API calls yet.** T-006-01 will connect layouts to the API. This ticket provides the plumbing (API client module) and the shell (layouts with placeholder content).

2. **Svelte 5 runes only.** No legacy `$:` reactive statements, no `writable()` stores. Use `$state()`, `$derived()`, `$effect()`.

3. **No new dependencies.** The ticket doesn't call for any npm packages beyond what's installed. The API client uses native `fetch` and `EventSource`/manual SSE parsing.

4. **Tailwind 4 utility classes.** No component library (no shadcn, no DaisyUI). Layouts are hand-styled with Tailwind utilities and brand theme variables.

5. **Cloudflare Pages deployment.** No Node.js APIs in client code. Server-side code (if any `+page.server.ts`) runs in CF Workers runtime.

6. **Project workspace tabs.** The `project/[id]` route needs tab navigation across 5 sub-routes: editor, materials, quote, viewer, export. This naturally maps to a `+layout.svelte` inside `project/[id]/`.

## Files & Modules Affected

### New files to create:
- `src/lib/api/` — API client module (apiFetch, SSE reader, error types, mock toggle)
- `src/lib/stores/` — Svelte 5 rune-based state (session, project)
- `src/lib/components/` — Shared layout components (sidebar, header, tab nav)
- `src/routes/+layout.svelte` — Rework to include sidebar + header
- `src/routes/dashboard/+layout.svelte` — Dashboard-specific layout shell
- `src/routes/catalog/+layout.svelte` — Catalog-specific layout shell
- `src/routes/project/[id]/+layout.svelte` — Project workspace with tab nav

### Files to modify:
- `src/routes/+layout.svelte` — Add app shell (sidebar, header)
- `src/routes/dashboard/+page.svelte` — Project list shell with create button
- `src/routes/catalog/+page.svelte` — Material list shell with add button
- `src/routes/project/[id]/+page.svelte` — Redirect or overview within tab layout
- `src/app.d.ts` — Uncomment/define App types if needed

### Files unchanged:
- `src/routes/+page.svelte` — Landing page stays independent of app shell
- `src/routes/c/[token]/+page.svelte` — Client-facing view has its own layout (no sidebar)
- `src/routes/settings/+page.svelte` — Inherits app shell, content stays placeholder
- All `project/[id]/{editor,materials,quote,viewer,export}/+page.svelte` — Stay placeholder

## Patterns to Follow

- **Svelte 5 runes for state:** `$state()` for mutable state, `$derived()` for computed values. Stores are just exported objects with `$state` fields — no `writable()`/`readable()`.
- **SvelteKit layouts:** Nested `+layout.svelte` files for shared chrome. `{@render children()}` for slot rendering.
- **`$lib` alias:** All shared code under `src/lib/`, imported as `$lib/api/client`, `$lib/stores/session`, etc.
- **Error types as classes:** `RateLimitError`, `AuthError`, `ValidationError` extending `Error` for `instanceof` checks.
- **Mock mode:** Check `import.meta.env.VITE_MOCK_API` at module level; swap real fetch for mock responses.

## Assumptions

1. Auth token will be a Bearer token stored in the session store (mechanism TBD — likely set after login, but login flow is out of scope).
2. The API returns JSON for all endpoints except SSE streaming endpoints.
3. SSE format matches `data: {json}\n\n` (standard SSE, matching what the CF Worker streams through).
4. Tenant name in the header is a placeholder string for now (no tenant API yet).
5. The landing page (`/`) and client view (`/c/[token]`) should NOT show the app sidebar/header — they have their own layouts.
