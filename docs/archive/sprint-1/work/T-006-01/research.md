# T-006-01 Research: Dashboard & Catalog Integration

## Scope

This ticket wires the SvelteKit frontend (T-005-03) to the Axum CRUD API (T-004-02) through the CF Worker proxy. It's the first vertical slice through the full stack.

---

## Frontend Current State

### Dashboard (`web/src/routes/(app)/dashboard/+page.svelte`)
- Static stub: "No projects yet" empty state, unbound "New Project" button.
- No data loading, no API calls, no reactive state wiring.

### Catalog (`web/src/routes/(app)/catalog/+page.svelte`)
- Static stub: "No materials in the catalog" empty state, unbound "Add Material" button.
- No data loading, no API calls.

### Project Page (`web/src/routes/(app)/project/[id]/+page.svelte`)
- Displays "Select a tab above to get started." — no project data loaded.
- Layout (`+layout.svelte`) shows "Project {id}" heading + TabNav. No API fetch.
- Layout load function (`+layout.ts`) passes `params.id` through — data-loading hook exists but only returns the raw ID.

### API Client (`web/src/lib/api/`)
- `client.ts`: `apiFetch<T>(path, options)` — typed fetch wrapper, base URL `/api`, injects `Authorization: Bearer` header from session store. Handles 401/422/429 error classes.
- `mock.ts`: `mockApiFetch<T>` — returns hardcoded data (3 projects, 4 materials, 2 zones) after 150-300ms delay. Mock project shape: `{ id, name, createdAt }`.
- `index.ts`: Conditional export — `VITE_MOCK_API=true` routes to mock, otherwise real client.
- `errors.ts`: `ApiError`, `RateLimitError`, `AuthError`, `ValidationError`.

### State Stores (`web/src/lib/stores/`)
- `session.svelte.ts`: Svelte 5 runes. Holds `tenant`, `authToken`, `activeProjectId`. Auth token defaults to empty string (no auth flow yet).
- `project.svelte.ts`: Holds `projects: Project[]`, `current: Project | null`, `zones: Zone[]`, `tiers: Tier[]`.
  - **Type mismatch**: Frontend `Project` has `{ id, name, createdAt }`. Backend `ProjectResponse` has `{ id, tenant_id, client_name, client_email, address, status, created_at, updated_at }`. The frontend type needs expansion.
  - **Type mismatch**: Frontend has no `Material` type at all. Backend `MaterialResponse` has 13 fields including `category`, `unit`, `price_per_unit`, `extrusion`, etc.

### Layout & Components
- `Sidebar.svelte`: Dashboard, Catalog, Settings nav. Active state via `$page.url.pathname`.
- `Header.svelte`: Shows tenant name from session store.
- `TabNav.svelte`: Project sub-pages (Editor, Materials, Quote, Viewer, Export).
- `ZoneEditor.svelte`: Canvas-based polygon drawing — not relevant to this ticket.

### Vite Dev Proxy (`web/vite.config.ts`)
- `/api` proxied to `API_URL || http://localhost:3000`. This means in dev mode, the frontend can talk to a local Axum instance directly. No CF Worker needed for local dev.

---

## Backend Current State

### API Routes (`crates/plantastic-api/src/routes/`)
All routes require `X-Tenant-Id` header (UUID). Returns JSON errors.

| Method | Path | Status | Notes |
|--------|------|--------|-------|
| POST | /projects | 201 | Returns `ProjectResponse` |
| GET | /projects | 200 | Returns `Vec<ProjectResponse>` |
| GET | /projects/{id} | 200 | Tenant-scoped |
| DELETE | /projects/{id} | 204 | Tenant-scoped |
| GET | /materials | 200 | Tenant-scoped |
| POST | /materials | 201 | Returns `MaterialResponse` |
| PATCH | /materials/{id} | 204 | Full replacement body |
| DELETE | /materials/{id} | 204 | Tenant-scoped |
| GET | /projects/{id}/zones | 200 | — |
| POST | /projects/{id}/zones | 201 | — |
| PUT | /projects/{id}/zones | 200 | Bulk replace |
| PATCH | /projects/{id}/zones/{zid} | 204 | — |
| DELETE | /projects/{id}/zones/{zid} | 204 | — |
| GET | /projects/{id}/tiers | 200 | — |
| PUT | /projects/{id}/tiers/{tier} | 204 | — |

### Backend Data Shapes
**ProjectResponse**: `{ id, tenant_id, client_name, client_email, address, status, created_at, updated_at }`
**MaterialResponse**: `{ id, tenant_id, name, category, unit, price_per_unit, depth_inches, extrusion, texture_key, photo_key, supplier_sku, created_at, updated_at }`

### Tenant Extraction (`crates/plantastic-api/src/extract.rs`)
- Reads `X-Tenant-Id` from request headers.
- Returns 400 if missing or not a valid UUID.
- V1 placeholder — no auth verification. Frontend must send this header.

---

## CF Worker Proxy (`worker/src/index.ts`)

### Critical Issue: Method Whitelist
The worker only allows `GET` and `POST`:
```typescript
if (request.method !== 'GET' && request.method !== 'POST') {
    return jsonError('Method not allowed', 405, env, requestOrigin);
}
```
But the API uses `PATCH`, `PUT`, and `DELETE` for materials, zones, and tiers. **The worker must be updated to allow all necessary HTTP methods.**

### CORS
`Access-Control-Allow-Methods: 'GET, POST, OPTIONS'` — also needs PATCH, PUT, DELETE.

### Headers Forwarded
Only `Content-Type` and `Authorization` are forwarded to the backend. **`X-Tenant-Id` is NOT forwarded.** This is another critical gap — the frontend sends `X-Tenant-Id`, but the worker strips it before proxying.

### Configuration
- `BACKEND_URL`: Not in `wrangler.toml` vars (will be a secret via `wrangler secret put`).
- `ALLOWED_ORIGIN`: Currently `*`.
- Rate limits: 60/min per IP, 200 lifetime per session.

---

## Environment Wiring Gaps

| Layer | Variable | Current State |
|-------|----------|---------------|
| Frontend (dev) | `API_URL` | Defaults to `http://localhost:3000` via vite proxy |
| Frontend (prod) | `PUBLIC_API_URL` | Not configured — vite proxy won't exist in prod |
| Worker | `BACKEND_URL` | Missing from wrangler.toml — must be a secret |
| Worker | Forwarded headers | Missing `X-Tenant-Id` |
| Frontend | `X-Tenant-Id` injection | Not implemented — client.ts doesn't send it |

---

## Scenario Targets

**S.INFRA.1 (Full Stack Round-Trip)**: Currently `NotImplemented`. The comment says "TARGET: This should turn green at end of Sprint 1 (T-006-01)." Steps include project CRUD, zone CRUD, material CRUD, tier assignment, and quote retrieval. Steps 7-9 (quote and delete) depend on capabilities not in this ticket's scope (pt-quote API route is T-008-01).

**S.INFRA.2 (Tenant Isolation)**: Currently `NotImplemented`. Tests cross-tenant data visibility. The backend logic exists (T-004-02 tests pass for isolation), but the scenario needs to be wired to run against the API.

---

## Key Findings

1. **Worker blocks PATCH/PUT/DELETE** — must fix method whitelist and CORS.
2. **Worker strips X-Tenant-Id** — must add to forwarded headers.
3. **Frontend has no Material type** — must add to project store or new store.
4. **Frontend Project type is thin** — `{ id, name, createdAt }` vs backend's full response. Needs `address`, `status`, `client_name`, etc.
5. **No tenant ID injection** in frontend `apiFetch` — must send `X-Tenant-Id` header.
6. **Mock data shapes don't match API** — mock projects have `name`, API has `client_name` + `address`. Mock materials have `category: 'tree'`, API has `category: 'hardscape'|'softscape'|'edging'|'fill'`.
7. **No loading/error states** exist in any page — all stubs are static HTML.
8. **S.INFRA.1 includes quote retrieval** (step 7) which depends on T-008-01. This ticket can advance S.INFRA.1 partially but not fully.
