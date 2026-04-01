# Plan — T-005-03: Route Skeleton & API Client

## Step 1: API Error Types

**Create** `web/src/lib/api/errors.ts`

- `ApiError` base class extending `Error` with `status` field.
- `RateLimitError` with `retryAfter`, `limit`, `remaining` — constructed from 429 response + headers.
- `AuthError` for 401 responses.
- `ValidationError` with `errors: Record<string, string[]>` for 422 responses.

**Verify:** TypeScript compiles, error classes instantiate correctly.

## Step 2: State Stores

**Create** `web/src/lib/stores/session.svelte.ts`
- Reactive state: `tenant`, `authToken`, `activeProjectId`.
- Getter/setter object export for cross-module reactivity.

**Create** `web/src/lib/stores/project.svelte.ts`
- Type definitions: `Project`, `Zone`, `Tier`.
- Reactive state: `projects`, `current`, `zones`, `tiers`.
- Reset function to clear project state.

**Verify:** Imports resolve, no TS errors.

## Step 3: API Client

**Create** `web/src/lib/api/client.ts`
- `apiFetch<T>(path, options?)`: constructs URL, injects `Authorization: Bearer {token}` from session store, sends request, parses JSON, inspects status codes, throws appropriate error class.
- `sseStream(path, options)`: fetch with streaming body reader, parses `data:` lines, calls `onEvent` callback, handles errors.
- Types: `ApiOptions` (method, body, headers), `SseOptions` (method, body, onEvent, onError, onDone).

**Verify:** TS compiles, no circular deps with stores.

## Step 4: Mock API

**Create** `web/src/lib/api/mock.ts`
- `mockApiFetch<T>`: path-based router returning mock data after artificial delay.
- `mockSseStream`: emits a few mock events then resolves.
- Mock data: 3 sample projects, 2 sample zones, basic material list.

**Create** `web/src/lib/api/index.ts`
- Check `import.meta.env.VITE_MOCK_API`.
- Re-export `apiFetch` and `sseStream` from either `client` or `mock`.
- Always re-export error classes from `errors`.

**Verify:** Import `$lib/api` resolves, mock toggle works conceptually.

## Step 5: Shared Components

**Create** `web/src/lib/components/Sidebar.svelte`
- Brand logo placeholder (text "Plantastic" or SVG).
- Nav links: Dashboard, Catalog, Settings.
- Active link highlighted using `$page.url.pathname`.
- Responsive: full sidebar on desktop, collapsible on mobile (stretch — basic version first).

**Create** `web/src/lib/components/Header.svelte`
- Horizontal bar. Tenant name from session store (fallback "Plantastic").
- User avatar placeholder (initials circle).

**Create** `web/src/lib/components/TabNav.svelte`
- Props: `projectId`.
- Tab items: Editor, Materials, Quote, Viewer, Export.
- Active tab based on `$page.url.pathname`.
- Horizontal scroll on small screens.

**Verify:** Components render without errors (tested in layout integration).

## Step 6: Route Restructure & Layouts

**Create** `web/src/routes/(app)/+layout.svelte`
- Import Sidebar, Header.
- Layout: sidebar on left, header on top of main area, children in main content area.
- Flexbox layout: sidebar fixed-width, main flexible.

**Move routes** into `(app)/`:
- `dashboard/+page.svelte` → `(app)/dashboard/+page.svelte` (update content)
- `catalog/+page.svelte` → `(app)/catalog/+page.svelte` (update content)
- `settings/+page.svelte` → `(app)/settings/+page.svelte` (move as-is)
- `project/[id]/` → `(app)/project/[id]/` (move entire directory)

**Create** `web/src/routes/(app)/project/[id]/+layout.svelte`
- Import TabNav.
- Render tab bar above children.
- Pass `data.id` as `projectId` prop to TabNav.

**Update** `(app)/dashboard/+page.svelte`:
- Project list shell: heading, empty state message, "New Project" button placeholder.

**Update** `(app)/catalog/+page.svelte`:
- Material list shell: heading, empty state message, "Add Material" button placeholder.

**Delete** old route files from pre-move locations.

**Verify:** All routes render at correct URLs. Sidebar shows on /dashboard, /catalog, /settings, /project/[id]/*. Landing page (/) and client view (/c/[token]) have NO sidebar.

## Step 7: App Types

**Update** `web/src/app.d.ts`:
- Uncomment and define `App.Error` with `message` and `status` fields.

**Verify:** `svelte-check` passes (or at minimum no new type errors).

## Step 8: Build Verification

- Run `cd web && npx svelte-check` to verify TypeScript.
- Verify the build works: `cd web && npm run build` (or at least check).

## Testing Strategy

This ticket is primarily layout/plumbing — no backend integration. Testing approach:

- **Type checking:** `svelte-check` validates all TypeScript and Svelte component types.
- **Build:** `npm run build` confirms production build succeeds with CF Pages adapter.
- **API client unit tests:** Not added in this ticket (no test framework in web/ yet — vitest would need to be added). The API client is exercised via mock mode when T-006-01 integrates it.
- **Visual verification:** Layouts render correctly (manual check during dev, or screenshot in review).

## Commit Strategy

1. Commit after Steps 1-4: "Add API client module with error types, stores, and mock mode"
2. Commit after Steps 5-7: "Add app shell layouts, components, and route restructure"
3. Commit after Step 8: "Verify build passes" (only if fixes needed)
