# T-006-01 Review: Dashboard & Catalog Integration

## Summary

This ticket wires the SvelteKit frontend to the Axum CRUD API through the CF Worker proxy, completing the first vertical slice through the entire stack: CF Pages → CF Worker → Lambda → PostGIS → response.

---

## Files Modified

| File | Change |
|------|--------|
| `worker/src/index.ts` | Expanded method whitelist to include PUT/PATCH/DELETE; updated CORS; added X-Tenant-Id header forwarding |
| `web/src/lib/stores/session.svelte.ts` | Added `tenantId` field with dev default UUID |
| `web/src/lib/api/client.ts` | Inject `X-Tenant-Id` header from session store in all requests |
| `web/src/lib/stores/project.svelte.ts` | Expanded `Project` interface to match backend; added `Material` interface |
| `web/src/lib/api/mock.ts` | Updated mock data shapes to match API; added CRUD handlers for all operations |
| `web/src/routes/(app)/dashboard/+page.svelte` | Full rewrite: API-driven project list, loading/error states, create modal |
| `web/src/routes/(app)/catalog/+page.svelte` | Full rewrite: API-driven material table, CRUD modal, delete with confirm |
| `web/src/routes/(app)/project/[id]/+page.svelte` | Full rewrite: project detail with zone count from API |
| `web/src/routes/(app)/project/[id]/+layout.svelte` | Reactive project name fetch via $effect |
| `.env.example` | Added VITE_MOCK_API and API_URL documentation |

## Acceptance Criteria Check

| Criterion | Status |
|-----------|--------|
| Dashboard calls GET /projects and renders project cards | Done |
| Create project: address → POST → redirect | Done |
| Project page calls GET /projects/:id with zone count | Done |
| Catalog calls GET /materials and renders list | Done |
| Add material: form → POST → appears in list | Done |
| Edit material: modal → PATCH → updates in list | Done |
| Delete material: confirm → DELETE → removed | Done |
| Environment wiring: CF Worker forwards to Lambda | Done (method whitelist + X-Tenant-Id) |
| Frontend sends tenant ID to API | Done (session store + apiFetch injection) |
| Full round-trip verified | Verified via mock mode; real API requires running backend |
| Error states: loading, empty, API errors | Done (all three pages) |

## Scenario Dashboard

**Before:** 20.0 / 240.0 min effective (8.3%), 4 pass, 13 not implemented
**After:** 20.0 / 240.0 min effective (8.3%), 4 pass, 13 not implemented

No scenario regression. No new scenarios flipped — this ticket delivers frontend integration (UI layer) which doesn't directly change computation scenarios. S.INFRA.1 and S.INFRA.2 remain `NotImplemented` because they require a running backend in CI, which is not part of this ticket's scope.

No new milestones claimed — the API and repository milestones were claimed by T-004-02 and T-003-02.

## Test Coverage

- **Type checking:** `svelte-check` — 313 files, 0 errors, 0 warnings
- **Production build:** Succeeds with CF Pages adapter
- **Worker type check:** `tsc --noEmit` passes
- **Quality gate:** `just check` passes (fmt + lint + test + scenarios)

### Test Gaps

1. **No vitest tests** for frontend components or API client. Frontend test infrastructure was noted as missing in T-005-03 review and remains outside this ticket's scope.
2. **No E2E tests** for the full round-trip (browser → worker → API → DB). Would require Playwright + running backend + database.
3. **Mock mode only** verified in this ticket. Real API integration depends on a deployed/running backend.

## Open Concerns

1. **Auth is still placeholder.** `X-Tenant-Id` is hardcoded in the session store. Any user can spoof any tenant. This is a known V1 limitation.
2. **Worker BACKEND_URL not configured.** The `wrangler.toml` doesn't include it — needs `wrangler secret put BACKEND_URL` before the worker is deployed.
3. **Material PATCH sends full body.** The backend's PATCH handler (`update_material`) requires all fields, not a partial update. The frontend sends all fields to satisfy this, but it means the edit modal must always show all fields.
4. **No pagination.** Both GET /projects and GET /materials return all rows. Fine for V1 with small datasets, but will need pagination for production.
5. **Double fetch in project pages.** Both the layout and the detail page fetch `/projects/:id`. This is a minor inefficiency — could be solved with a shared cache or layout-level data passing.
6. **S.INFRA.1 still NotImplemented.** Flipping this scenario requires a CI-compatible integration test that talks to a real backend. This is infrastructure work, not frontend work.
