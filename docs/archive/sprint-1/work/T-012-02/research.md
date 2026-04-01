# T-012-02 Research — Catalog Search & Filter

## Ticket goal

Add client-side search and category filtering to the material catalog page. Make the search/filter logic reusable so it can be embedded in the MaterialPicker component during zone assignment. Advance S.2.2 from OneStar to TwoStar.

## Existing code inventory

### Frontend — Catalog page
- **`web/src/routes/(app)/catalog/+page.svelte`** (323 lines): Full CRUD page. Loads materials via `apiFetch<Material[]>('/materials')`. Renders a table with name, category badge, unit, price, edit/delete buttons. Modal form for create/edit. No search, no filtering, no category tabs.
- **Category colors** already defined at line 130: `categoryColors` record mapping hardscape/softscape/edging/fill to Tailwind badge classes.
- **`formatPrice()`** and **`formatUnit()`** helper functions already exist (lines 115–128).

### Frontend — MaterialPicker
- **`web/src/lib/components/assignment/MaterialPicker.svelte`** (101 lines): Used in the materials assignment page. Groups materials by category using `CATEGORY_ORDER` and `CATEGORY_LABELS` constants. No search input. Takes `materials` as a prop — doesn't fetch its own data.
- Already has category grouping logic in a `$derived.by()` block (line 45–52).
- Duplicates `formatUnit()` and `formatPrice()` from the catalog page.

### Frontend — Types
- **`web/src/lib/stores/project.svelte.ts`**: Exports `Material` interface with `category: 'hardscape' | 'softscape' | 'edging' | 'fill'` and `name: string` — the two fields we need for search/filter.
- **`web/src/lib/api/mock.ts`**: 4 mock materials, one per category. Enough to test filtering.

### Backend — pt-materials crate
- **`crates/pt-materials/src/types.rs`**: `MaterialCategory` enum with `Hardscape/Softscape/Edging/Fill`, serde `rename_all = "snake_case"`. `Material` struct with `name: String` and `category: MaterialCategory`.
- **`crates/pt-materials/src/builder.rs`**: Builder pattern. No search/filter logic — that's all frontend for V1.
- **No backend changes needed.** The ticket says "client-side for V1, debounced".

### Scenario test
- **`tests/scenarios/src/suites/design.rs`** line 146–278: `s_2_2_material_catalog()` currently at OneStar. Tests domain model serde and category filtering. Comment at line 277 says path to TwoStar is "T-012-02 adds search/filter tested via the API layer."
- The scenario currently builds 5 materials in Rust and filters by category. To reach TwoStar, we need to test that search/filter works through the API layer or demonstrate the component contract.

### Scenario milestone
- **`tests/scenarios/src/progress.rs`** line 106–117: pt-materials milestone already claimed by T-012-01. Note says "Path to TwoStar: T-012-02 adds search/filter." No new milestone needed — this ticket advances the existing scenario.

## Patterns and conventions observed

1. **Svelte 5 runes**: The codebase uses `$state`, `$derived`, `$props()`. No Svelte 4 stores.
2. **Component props**: Components use `$props()` with destructured typed parameters.
3. **API layer**: `apiFetch` from `$lib/api` handles auth headers and JSON. Mock mode via `VITE_MOCK_API`.
4. **Tailwind CSS**: All styling via Tailwind utility classes. Brand colors: `brand-primary`, `brand-secondary`, `brand-accent`.
5. **No shared utility file** for format helpers — `formatUnit()` and `formatPrice()` are duplicated between catalog page and MaterialPicker.

## Constraints and assumptions

1. **Client-side filtering** — the ticket explicitly says V1 is client-side with debounce. No new API endpoints needed.
2. **Reusable component** — the search/filter must be embeddable in MaterialPicker later. This means extracting it as a standalone component, not inline in the catalog page.
3. **Category counts** — the ticket requires showing material counts per category (e.g., "Hardscape (2)").
4. **S.2.2 TwoStar** — the scenario test needs to verify more than just the domain model. TwoStar means "tested via the API layer." Since this is a frontend feature, we can advance by testing the filtering/search logic in a Rust-side test that exercises the `MaterialCategory` filtering contract the frontend depends on, plus name-based text search matching.
5. **No new Rust crate code needed** for the actual search/filter — it's all Svelte. But the scenario test should verify the contract the frontend relies on.

## Key decisions to make in Design

1. Extract search/filter as a reusable Svelte component vs. inline logic with exported functions?
2. Where to put shared format helpers (formatUnit, formatPrice)?
3. How to advance S.2.2 to TwoStar — what specifically to test?
4. Debounce strategy: built-in vs. tiny utility.
