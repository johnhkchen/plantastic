# T-006-01 Structure: Dashboard & Catalog Integration

## Files Modified

### 1. CF Worker — Method Whitelist & Header Forwarding
**`worker/src/index.ts`**
- Expand method check from `GET|POST` to `GET|POST|PUT|PATCH|DELETE`.
- Update CORS `Access-Control-Allow-Methods` to include `PUT, PATCH, DELETE`.
- Add `X-Tenant-Id` to forwarded headers in `proxyToBackend()`.

### 2. Frontend API Client — Tenant ID Injection
**`web/src/lib/api/client.ts`**
- In `apiFetch()`, add `X-Tenant-Id` header from session store alongside existing auth header injection.
- Applies to all requests automatically.

### 3. Frontend Session Store — Dev Tenant ID
**`web/src/lib/stores/session.svelte.ts`**
- Add a default `tenantId` field (UUID string). For V1 dev, hardcode a known UUID that matches test fixtures. Can be overridden by auth flow later.

### 4. Frontend Project Types — Expand to Match API
**`web/src/lib/stores/project.svelte.ts`**
- Expand `Project` interface: add `tenant_id`, `client_name`, `client_email`, `address`, `status`, `updated_at`. Remove `name` (backend uses `client_name` + `address`).
- Keep `Zone` and `Tier` interfaces as-is (not used in this ticket's pages).

### 5. Frontend Mock Data — Update Shapes
**`web/src/lib/api/mock.ts`**
- Update `MOCK_PROJECTS` to match expanded `Project` interface.
- Update `MOCK_MATERIALS` to match backend `MaterialResponse` shape.
- Add mock handlers for POST/PATCH/DELETE operations (return success responses).

## Files Created

### 6. Dashboard Page — Full Rewrite
**`web/src/routes/(app)/dashboard/+page.svelte`**
- Script: `$state` for `projects`, `loading`, `error`, `showCreateModal`.
- `$effect`: On mount, call `apiFetch<Project[]>('/projects')`, populate store.
- Project cards: Loop over projects, show client_name/address/status/created_at.
- Empty state: Shown when `projects.length === 0 && !loading`.
- Loading state: Skeleton cards while fetching.
- Error state: Banner with retry button.
- "New Project" button: Opens create modal.
- Create modal: Form with address, client_name, client_email inputs. POST /projects, on success push to list + close modal (or `goto` project page).

### 7. Catalog Page — Full Rewrite
**`web/src/routes/(app)/catalog/+page.svelte`**
- Script: `$state` for `materials`, `loading`, `error`, `showModal`, `editingMaterial`.
- `$effect`: On mount, call `apiFetch<Material[]>('/materials')`.
- Material table/cards: name, category, unit, price_per_unit.
- Empty state, loading skeleton, error banner (same pattern as dashboard).
- "Add Material" button: Opens modal.
- Material modal: Form with name, category (select), unit (select), price_per_unit, depth_inches, extrusion (defaults). Shared for create + edit.
- Create: POST /materials → append to list.
- Edit: Click row → open modal pre-filled → PATCH /materials/:id → update in list.
- Delete: Click delete → browser confirm() → DELETE /materials/:id → remove from list.

### 8. Project Detail Page — Wire Data Loading
**`web/src/routes/(app)/project/[id]/+page.svelte`**
- Replace stub with project detail display.
- `$effect`: Fetch `apiFetch<Project>(\`/projects/${id}\`)`.
- Display: client_name, address, status, created_at, zone count.
- Zone count: Fetch `apiFetch<Zone[]>(\`/projects/${id}/zones\`)` and display count.
- Loading/error states.

### 9. Project Layout — Show Real Name
**`web/src/routes/(app)/project/[id]/+layout.svelte`**
- Fetch project on layout load, display `client_name || address || "Untitled"` instead of raw UUID.

## Module Boundaries

```
┌──────────────────────────────┐
│  Dashboard Page              │
│  - fetches /projects         │
│  - manages create modal      │
│  - updates projectStore      │
├──────────────────────────────┤
│  Catalog Page                │
│  - fetches /materials        │
│  - manages CRUD modal        │
│  - local material state      │
├──────────────────────────────┤
│  Project Detail Page         │
│  - fetches /projects/:id     │
│  - fetches /projects/:id/zones│
│  - displays summary          │
├──────────────────────────────┤
│  API Client (client.ts)      │
│  - injects X-Tenant-Id       │
│  - existing apiFetch/errors  │
├──────────────────────────────┤
│  Session Store               │
│  + tenantId: string          │
├──────────────────────────────┤
│  Project Store               │
│  - expanded Project type     │
├──────────────────────────────┤
│  CF Worker                   │
│  - expanded methods          │
│  - forwards X-Tenant-Id      │
└──────────────────────────────┘
```

## Ordering Constraints

1. Worker fix (methods + headers) must come first — without it, PATCH/PUT/DELETE are 405.
2. Session store `tenantId` + client.ts injection must come before any page can make real API calls.
3. Type expansions before page rewrites (pages depend on correct types).
4. Mock data update alongside type expansion (keeps mock mode working).
5. Dashboard and catalog pages are independent — can be done in either order.
6. Project detail page depends on having at least one project created (tested via dashboard).

## Files NOT Changed

- `web/src/lib/components/Sidebar.svelte` — no changes needed.
- `web/src/lib/components/Header.svelte` — no changes needed.
- `web/src/lib/components/TabNav.svelte` — no changes needed.
- `web/src/routes/(app)/+layout.svelte` — no changes needed.
- Backend Rust code — all routes exist, no backend changes required.
- Database migrations — schema is complete.
- `web/src/lib/api/errors.ts` — error classes are sufficient.
- `web/src/lib/api/index.ts` — conditional export logic unchanged.
