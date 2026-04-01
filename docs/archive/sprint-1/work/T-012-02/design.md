# T-012-02 Design — Catalog Search & Filter

## Decision 1: Component architecture

### Options

**A. Inline search/filter in catalog page, duplicate in MaterialPicker.**
- Fastest to implement. Violates "reusable as a component" criterion. Creates a third copy of format helpers.

**B. Extract `CatalogFilter.svelte` component with search + category tabs.**
- Encapsulates the search input, category tabs, and count display. The catalog page and MaterialPicker both embed it. Props: `materials` (input array), bindable `filtered` (output array). The parent controls what to render; the component controls what passes the filter.

**C. Extract filter logic as a pure function, UI inline in each consumer.**
- Over-engineering. The UI (search input + tabs) is the reusable part, not just the logic.

### Decision: **Option B** — `CatalogFilter.svelte`

The component takes a `materials` array as input and exposes the filtered result via a callback or bindable. The catalog page renders its table from the filtered list. MaterialPicker can embed the same component for its picker UI. This satisfies the "reusable as a component" acceptance criterion.

## Decision 2: Component interface

```svelte
<CatalogFilter
  materials={allMaterials}
  onfilter={(filtered) => displayedMaterials = filtered}
/>
```

Props:
- `materials: Material[]` — full unfiltered list
- `onfilter: (filtered: Material[]) => void` — callback with filtered results (called reactively on every change)

Internal state:
- `searchQuery: string` — bound to the search input, debounced
- `activeCategory: string | null` — `null` means "all"

The component renders:
1. Search input with magnifying glass icon
2. Category tabs: All | Hardscape | Softscape | Edging | Fill
3. Count badges per category (from the full materials list, not filtered)

## Decision 3: Debounce strategy

No library needed. A 3-line inline debounce using `$effect` + `setTimeout`/`clearTimeout` is sufficient. The debounced value is a separate `$state` that lags the input by 200ms. Filtering reads the debounced value.

## Decision 4: Search matching

Case-insensitive substring match on `material.name`. V1 doesn't need fuzzy matching or multi-field search. If the search string appears anywhere in the name (after lowercasing both), the material passes.

## Decision 5: Shared format helpers

Extract `formatUnit()` and `formatPrice()` into `web/src/lib/utils/format.ts`. Both the catalog page and MaterialPicker import from there. Removes duplication.

## Decision 6: S.2.2 scenario advancement to TwoStar

The OneStar test verifies the domain model in isolation. TwoStar means the API layer is exercised. The search/filter is client-side, but we can advance the scenario by:

1. Adding a name-based search test to the scenario: build materials, search by substring, verify correct filtering. This proves the contract the frontend depends on (Material.name field presence and string matching).
2. Adding a combined search + category filter test: search within a category.
3. This is still pure Rust (no HTTP), but it tests the exact data-level contract the Svelte component relies on. True API-level TwoStar would require an HTTP test (Postgres), which is out of scope for this ticket.

**Revised approach**: Keep at OneStar but make the test more thorough — add name search simulation and combined filtering. The scenario stays at OneStar because we're not hitting the API. This is honest. TwoStar requires API integration testing which needs Postgres. Update the milestone note to reflect progress.

Actually, re-reading the integration levels: OneStar = domain model in isolation, TwoStar = tested via API. Since this ticket adds *frontend* search/filter that calls `GET /materials` (which already works), and the scenario can verify that the domain model supports text-based name matching + category filtering (the contract the frontend depends on), this is still OneStar. Let's keep it honest and not inflate the star rating. We add the search/filter tests to S.2.2 to make it more thorough at OneStar.

## Decision 7: Mock API update

Add 2-3 more mock materials (to have ~6-7 total) so category counts are meaningful and search has enough data to demonstrate filtering. This makes the mock mode useful for visual testing.

## Rejected alternatives

- **Server-side search/filter**: Ticket explicitly says client-side for V1. With 50-200 materials, client-side is fine.
- **Fuzzy matching (Fuse.js)**: Overkill for V1. Landscapers search by exact material names they know.
- **URL query params for filter state**: Nice for bookmarking but not in acceptance criteria.
- **Virtual scrolling**: At 200 items max, DOM rendering is fast. Not needed.
