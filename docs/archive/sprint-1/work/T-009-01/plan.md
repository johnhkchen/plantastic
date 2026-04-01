# T-009-01 Plan: Material Assignment UI

## Step 1: API Client Modules

Create `web/src/lib/api/tiers.ts` and `web/src/lib/api/quotes.ts` with typed fetch wrappers.

**Verification**: TypeScript compiles without errors.

## Step 2: Mock API Handlers

Add tier and quote mock handlers to `web/src/lib/api/mock.ts`:
- In-memory `mockTierAssignments` map keyed by `projectId:tier`
- `GET /projects/:id/tiers` returns 3 tiers from mock store
- `PUT /projects/:id/tiers/:tier` stores assignments
- `GET /projects/:id/quote/:tier` computes line items from mock zones + materials + assignments

**Verification**: Mock handlers return expected shapes.

## Step 3: Assignment Components

Build four Svelte components in `web/src/lib/components/assignment/`:

### 3a: TierTabs.svelte
- Three buttons: Good, Better, Best
- Active tier has underline accent
- Bindable `activeTier` prop

### 3b: ZoneList.svelte
- Renders zones as clickable rows
- Shows type badge (colored), area, perimeter, assigned material name
- Selected zone gets highlight border + background
- Bindable `selectedZoneId`

### 3c: MaterialPicker.svelte
- Groups materials by category with section headers
- Each material: name, price/unit, clickable
- Checkmark on currently assigned material
- Disabled overlay when no zone selected
- Fires `onAssign(materialId)` callback

### 3d: QuoteSummary.svelte
- Shows tier name, subtotal, total
- Line items: zone label, material name, quantity, unit price, line total
- Loading skeleton state
- Empty state when no assignments

**Verification**: Components render correctly with sample props.

## Step 4: Materials Page Orchestrator

Replace `web/src/routes/(app)/project/[id]/materials/+page.svelte`:

- On mount: fetch zones, materials, all tiers in parallel
- Three-column layout with components from Step 3
- Assignment flow: zone select → material click → update local state → debounced save → quote refresh
- Debounce: 800ms after last assignment change, PUT tier, then GET quote
- Error handling: error banner with retry

**Verification**: Full workflow works in dev mode with mock API:
1. Navigate to Materials tab
2. See zone list populated
3. Click a zone → highlight
4. Switch tiers → tabs work
5. Click a material → assignment appears on zone row
6. Quote summary updates after save

## Step 5: Polish & Edge Cases

- Empty states: no zones ("Draw zones in the Editor tab first"), no materials ("Add materials in the Catalog")
- Loading states for initial fetch
- Error states for save failures
- Unassign: click the currently assigned material to remove assignment
- Responsive: stack panels vertically on narrow screens (optional for V1)

**Verification**: Edge cases handled gracefully.

## Step 6: Run Quality Gate

- `just fmt` — auto-format
- `just lint` — clippy (Rust only, but verify no regressions)
- `just test` — workspace tests pass
- `just scenarios` — dashboard runs, no regressions

**Verification**: `just check` passes. Scenario dashboard shows same or higher numbers.

## Testing Strategy

This is a frontend-only ticket. All backend endpoints already exist and are tested.

- **No new Rust tests** — backend is unchanged
- **No new scenario tests** — this ticket enables T-009-02 but doesn't flip scenarios
- **Manual verification** via dev server with mock API
- **Existing tests must not regress**: `just test` and `just scenarios` must pass
