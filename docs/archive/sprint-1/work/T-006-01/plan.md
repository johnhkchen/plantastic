# T-006-01 Plan: Dashboard & Catalog Integration

## Step 1: Fix CF Worker — Methods + Headers
**Files:** `worker/src/index.ts`
**Changes:**
- Expand method whitelist: `GET|POST|PUT|PATCH|DELETE`
- Update CORS `Access-Control-Allow-Methods` to include all five methods
- Forward `X-Tenant-Id` header in `proxyToBackend()`
**Verification:** Worker builds (`cd worker && npx tsc --noEmit`). Existing worker tests pass.

## Step 2: Add Tenant ID to Session Store + API Client
**Files:** `web/src/lib/stores/session.svelte.ts`, `web/src/lib/api/client.ts`
**Changes:**
- Add `tenantId` field to session store with a dev default UUID
- In `apiFetch()`, inject `X-Tenant-Id` header from `session.tenantId`
**Verification:** `npx svelte-check` passes.

## Step 3: Expand Frontend Types
**Files:** `web/src/lib/stores/project.svelte.ts`
**Changes:**
- Expand `Project` interface to match backend `ProjectResponse`
- Add `Material` interface matching backend `MaterialResponse`
- Export both types for use in pages
**Verification:** `npx svelte-check` passes (existing references to `Project.name` will break — fix in same step).

## Step 4: Update Mock Data
**Files:** `web/src/lib/api/mock.ts`
**Changes:**
- Update `MOCK_PROJECTS` shape: `client_name`, `address`, `status`, `created_at`, `updated_at`
- Update `MOCK_MATERIALS` shape: `category`, `unit`, `price_per_unit`, `extrusion`, etc.
- Add mock handlers for POST (create), PATCH (update), DELETE operations
- Update mock router to handle method-based routing
**Verification:** `npx svelte-check` passes. Mock mode still works.

## Step 5: Build Dashboard Page
**Files:** `web/src/routes/(app)/dashboard/+page.svelte`
**Changes:**
- Full rewrite: loading → fetch projects → render cards or empty state
- Project cards: client_name (or "Untitled"), address, status badge, date
- Create project modal: address, client_name, client_email fields
- POST /projects on submit → push to list or goto project page
- Error state with retry
**Verification:** Page renders with mock data (`VITE_MOCK_API=true`). `npx svelte-check` passes.

## Step 6: Build Catalog Page
**Files:** `web/src/routes/(app)/catalog/+page.svelte`
**Changes:**
- Full rewrite: loading → fetch materials → render table or empty state
- Material rows: name, category, unit, price
- Create/edit modal: name, category (select), unit (select), price, depth, extrusion type
- POST /materials, PATCH /materials/:id, DELETE /materials/:id
- Confirm before delete
- Error state with retry
**Verification:** Page renders with mock data. `npx svelte-check` passes.

## Step 7: Build Project Detail Page
**Files:** `web/src/routes/(app)/project/[id]/+page.svelte`, `web/src/routes/(app)/project/[id]/+layout.svelte`
**Changes:**
- Page: fetch project + zones, display detail summary (client, address, status, zone count)
- Layout: fetch project, show real name instead of UUID
- Loading/error states
**Verification:** Navigating to `/project/<uuid>` shows project detail (with mock or real API).

## Step 8: Update .env.example + Documentation
**Files:** `.env.example`
**Changes:**
- Add `VITE_MOCK_API=true` (for frontend dev without backend)
- Add comment about `X-Tenant-Id` / dev tenant setup
**Verification:** File updated.

## Step 9: Run Quality Gate
**Command:** `just check` (fmt + lint + test + scenarios)
**Verification:** All checks pass. Scenario dashboard shows no regressions.

---

## Testing Strategy

**Type checking:** `npx svelte-check` after every step — catches type mismatches early.
**Mock mode:** Test all pages with `VITE_MOCK_API=true` to verify UI logic without a running backend.
**Build:** `npm run build` in `web/` — production build must succeed (CF Pages adapter).
**Worker:** `cd worker && npx tsc --noEmit` — type check worker changes.
**Quality gate:** `just check` at the end for Rust side (no Rust changes expected, but verify no regressions).

No new Rust tests in this ticket — the backend is unchanged. Frontend testing infrastructure (vitest) was noted as missing in T-005-03 review; adding it is outside this ticket's scope.
