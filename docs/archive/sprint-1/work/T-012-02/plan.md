# T-012-02 Plan — Catalog Search & Filter

## Step 1: Create format utility module

Create `web/src/lib/utils/format.ts` with `formatUnit()` and `formatPrice()`. These are exact copies of the existing implementations, moved to a shared location.

**Verify:** Import works — no type errors.

## Step 2: Create CatalogFilter component

Create `web/src/lib/components/catalog/CatalogFilter.svelte`:
- Props: `materials: Material[]`, `onfilter: (filtered: Material[]) => void`
- Search input: `$state` for raw query, debounced via `$effect` + setTimeout (200ms)
- Category tabs: All, Hardscape, Softscape, Edging, Fill — each shows count from full materials list
- `$effect` watches debouncedQuery + activeCategory, filters materials, calls `onfilter()`
- Category color mapping for tab styling (moved from catalog page)

**Verify:** Component renders without errors in isolation.

## Step 3: Integrate CatalogFilter into catalog page

Modify `web/src/routes/(app)/catalog/+page.svelte`:
- Import CatalogFilter and format utils
- Remove local formatUnit, formatPrice, categoryColors
- Add `filteredMaterials` state, initialized from `materials`
- Place CatalogFilter between header and table
- Render table from filteredMaterials instead of materials
- Show "Showing X of Y materials" count when filtering is active

**Verify:** Page renders, search filters in real-time, category tabs filter, combined filtering works.

## Step 4: Update MaterialPicker imports

Modify `web/src/lib/components/assignment/MaterialPicker.svelte`:
- Remove local `formatUnit()` and `formatPrice()`
- Import from `$lib/utils/format`

**Verify:** No functional change — MaterialPicker renders identically.

## Step 5: Expand mock materials

Add 3 materials to `web/src/lib/api/mock.ts`:
- "River Rock" (fill, cu_yd)
- "Lavender" (softscape, each)
- "Concrete Pavers" (hardscape, sq_ft)

**Verify:** Mock API returns 7 materials, all categories have ≥1.

## Step 6: Advance S.2.2 scenario test

Add to `s_2_2_material_catalog()` in `tests/scenarios/src/suites/design.rs`:
- Name search test: filter catalog where name contains "pav" (case-insensitive) → should match "Travertine Pavers" and "Flagstone" (no — Flagstone doesn't contain "pav"). Just "Travertine Pavers". So filter for "pav" → 1 result, filter for "steel" → 1 result (Steel Edging).
- Combined filter: category=Hardscape AND name contains "flag" → 1 result (Flagstone)
- Empty search: "" matches all
- No-match search: "nonexistent" → 0 results
- Keep outcome at OneStar with updated comment about what T-012-02 delivered

**Verify:** `cargo run -p pt-scenarios` — S.2.2 still passes, no regressions.

## Step 7: Run quality gate

`just check` — fmt, lint, test, scenarios all pass.

## Testing strategy

- **Scenario test (S.2.2)**: Expanded with search/filter assertions at domain model level.
- **Manual verification**: With `VITE_MOCK_API=true`, the catalog page shows search input, category tabs with counts, and real-time filtering.
- **No new unit tests**: The search/filter is pure UI logic (substring match + equality check) — trivial to verify by inspection. The scenario test covers the data contract.
