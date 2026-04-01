# T-006-01 Design: Dashboard & Catalog Integration

## Decision 1: Frontend Data Types — Align with Backend DTOs

### Options
A. **Expand existing types** in `project.svelte.ts`, add `Material` type there too.
B. **Create separate type files** (`types/project.ts`, `types/material.ts`) imported by stores and pages.
C. **Auto-generate from OpenAPI** — generate TS types from API schema.

### Decision: Option A
Keep types co-located with stores. The project store already holds `Project` and `Zone` — adding `address`, `status`, etc. to `Project` and adding a `Material` interface is minimal churn. No need for a separate types layer at this scale. Mock data shapes get updated to match.

Option C is premature — no OpenAPI spec exists and generating one is out of scope.

---

## Decision 2: Tenant ID Injection Strategy

### Options
A. **Frontend injects X-Tenant-Id** in `apiFetch()` from session store, worker forwards it.
B. **Worker injects X-Tenant-Id** from auth token claims.
C. **Hardcode tenant ID** in worker env vars (V1 single-tenant shortcut).

### Decision: Option A
The session store already holds tenant context. Auth is a placeholder (V1) — there's no JWT to extract claims from. The `apiFetch` function already injects `Authorization`; adding `X-Tenant-Id` is the same pattern. The worker just needs to forward it.

For V1, the frontend will use a hardcoded dev tenant ID stored in the session store. This matches the backend's current design (header-based, no auth verification).

---

## Decision 3: CF Worker Method Whitelist Fix

### Options
A. **Expand whitelist** to include GET, POST, PUT, PATCH, DELETE.
B. **Remove whitelist entirely** — pass all methods through.
C. **Use POST-tunneling** — frontend sends POST with `X-HTTP-Method-Override` header.

### Decision: Option A
Explicit whitelist is safer than wide-open. POST-tunneling adds unnecessary complexity. The set of needed methods is known: GET, POST, PUT, PATCH, DELETE. CORS `Access-Control-Allow-Methods` updated to match.

---

## Decision 4: Data Loading Pattern

### Options
A. **SvelteKit load functions** (`+page.ts` or `+page.server.ts`) — data fetched before render.
B. **Component-level `$effect`** — fetch on mount via reactive effects.
C. **Hybrid** — use load functions for initial data, effects for mutations.

### Decision: Option B (component-level effects)
SvelteKit load functions (`+page.ts`) run universally (SSR + client). Our API requires `X-Tenant-Id` and auth headers that only exist client-side. Using `+page.server.ts` would mean the SvelteKit server calls the API, but in production (CF Pages), there's no Node server — it's static/edge. Load functions would need to handle the CF Pages adapter's constraints.

Component-level `$effect` with `onMount`-style guards is simpler: the page renders a loading skeleton, fires `apiFetch` on mount, then replaces with data. This matches the existing Svelte 5 rune patterns in the codebase (see `ZoneEditor.svelte`).

Mutations (create, edit, delete) use `apiFetch` directly from event handlers, then update local state.

---

## Decision 5: Material Store Separation

### Options
A. **Add materials to projectStore** alongside projects/zones/tiers.
B. **Create a new `materialStore`** in `stores/material.svelte.ts`.
C. **No global store** — manage material state locally in the catalog page.

### Decision: Option C (local page state) with store for cross-page use
Materials are primarily used in the catalog page. The catalog page manages its own `materials` array via `$state`. For the project material assignment page (future ticket T-009-01), a store would be needed. For now, keep it simple: catalog page owns material state.

Projects list uses the existing `projectStore.projects` since it's already there and may be referenced from other pages (e.g., the sidebar count).

---

## Decision 6: Error and Loading State Pattern

### Options
A. **Per-page inline states** — each page manages its own `loading`, `error` state vars.
B. **Shared `AsyncState<T>` wrapper** component.
C. **SvelteKit error pages** (`+error.svelte`).

### Decision: Option A
Three pages need loading/error states (dashboard, catalog, project detail). Each has different shapes. A shared wrapper is premature abstraction for 3 consumers. Each page gets:
- `let loading = $state(true)` — shows skeleton/spinner
- `let error = $state<string | null>(null)` — shows error banner
- Data fetch in `$effect` sets `loading = false` and populates data or sets error.

---

## Decision 7: Create Project Flow

### Options
A. **Modal dialog** with form fields.
B. **Separate /new route** with full-page form.
C. **Inline form** that expands in the dashboard.

### Decision: Option A (modal)
The create project form is minimal (address, client name, client email — all optional per the API). A full page is overkill. A modal keeps the user on the dashboard and shows the new project in the list immediately after creation. The modal pattern also works well for "Add Material" in the catalog.

---

## Decision 8: Inline Edit for Materials

### Options
A. **Inline row editing** — click to edit cells in-place.
B. **Edit modal** — click opens modal pre-filled with material data.
C. **Separate edit page** per material.

### Decision: Option B (edit modal)
Inline editing of 9+ fields in a table row is poor UX. A modal pre-filled with the material's current values is cleaner and reusable (same form for create and edit). Delete uses a confirm dialog (browser `confirm()` for V1, no custom dialog needed).

---

## Decision 9: What Happens to Mock Mode

Mock mode (`VITE_MOCK_API=true`) stays as a fallback for offline development. The mock data shapes will be updated to match the real API response shapes. No other changes needed — the conditional export in `index.ts` handles the toggle.

---

## Scenario Impact

**S.INFRA.1**: This ticket enables steps 1-6 from the UI side. Steps 7-9 (quote retrieval, project delete confirmation via API) depend on T-008-01. The scenario test itself runs against the API directly (not through the UI), so it could be partially wired in this ticket by implementing steps 1-6 as a Rust integration test. However, S.INFRA.1 is an all-or-nothing scenario — partial implementation stays `NotImplemented`.

**S.INFRA.2**: Same situation — the backend logic works (tested in T-004-02), but the scenario test needs API-level assertions. Can be partially wired but won't flip green until the full stack is testable in CI.

This ticket does not directly flip any scenario. It delivers the **frontend integration layer** that makes the existing API capabilities reachable by users. No new milestones to claim — the API milestone was claimed by T-004-02.
