# Progress — T-005-03: Route Skeleton & API Client

## Status: Complete

### Step 1: API Error Types — DONE
Created `web/src/lib/api/errors.ts` with ApiError, RateLimitError, AuthError, ValidationError.

### Step 2: State Stores — DONE
Created `web/src/lib/stores/session.svelte.ts` and `project.svelte.ts` using Svelte 5 runes.

### Step 3: API Client — DONE
Created `web/src/lib/api/client.ts` with `apiFetch<T>` and `sseStream` functions.

### Step 4: Mock API — DONE
Created `web/src/lib/api/mock.ts` with mock data and `index.ts` barrel with VITE_MOCK_API toggle.

### Step 5: Shared Components — DONE
Created Sidebar.svelte, Header.svelte, TabNav.svelte in `web/src/lib/components/`.

### Step 6: Route Restructure & Layouts — DONE
- Created `(app)` route group with shared layout (sidebar + header).
- Moved dashboard, catalog, settings, project routes into `(app)/`.
- Created `(app)/project/[id]/+layout.svelte` with tab navigation.
- Updated dashboard page with project list shell + "New Project" button.
- Updated catalog page with material list shell + "Add Material" button.
- Deleted old route directories.

### Step 7: App Types — DONE
Updated `app.d.ts` with `App.Error` interface.

### Step 8: Build Verification — DONE
- `svelte-check`: 306 files, 0 errors, 0 warnings.
- `npm run build`: Production build succeeds with CF Pages adapter.

## Deviations from Plan
- Added `$lib/index.ts` barrel re-exporting API client, stores for convenience.
- No separate commits per step grouping — all changes are uncommitted (Lisa handles commits).
