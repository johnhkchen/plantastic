# T-012-02 Structure — Catalog Search & Filter

## Files to create

### `web/src/lib/components/catalog/CatalogFilter.svelte`
New reusable component. Props: `materials: Material[]`, `onfilter: (filtered: Material[]) => void`. Internal state: search query (debounced), active category. Renders: search input, category tab bar with counts, emits filtered list on change. ~120 lines.

### `web/src/lib/utils/format.ts`
New utility module. Exports `formatUnit(unit: string): string` and `formatPrice(price: string): string`. Eliminates duplication between catalog page and MaterialPicker.

## Files to modify

### `web/src/routes/(app)/catalog/+page.svelte`
- Import and embed `CatalogFilter` above the table
- Remove inline `formatUnit()` and `formatPrice()` — import from `$lib/utils/format`
- Remove `categoryColors` (moved to CatalogFilter)
- Add `filteredMaterials` state, render table from filtered list instead of raw `materials`
- Add material count display (e.g. "Showing 3 of 12 materials")
- ~30 lines changed, ~20 lines removed

### `web/src/lib/components/assignment/MaterialPicker.svelte`
- Import `formatUnit` and `formatPrice` from `$lib/utils/format` instead of defining locally
- Remove local `formatUnit()` and `formatPrice()` functions
- ~4 lines changed

### `web/src/lib/api/mock.ts`
- Add 3 more mock materials to bring total to 7: "River Rock" (fill), "Lavender" (softscape), "Concrete Pavers" (hardscape)
- This gives meaningful counts: hardscape(3), softscape(2), edging(1), fill(2)

### `tests/scenarios/src/suites/design.rs`
- In `s_2_2_material_catalog()`: add assertions for name-based filtering (case-insensitive substring match) and combined category+search filtering
- Keep at OneStar — domain model verification, not API-level
- ~20 lines added

## Files NOT modified

- `crates/pt-materials/src/**` — no Rust domain changes needed
- `tests/scenarios/src/progress.rs` — milestone already claimed by T-012-01; no new milestone
- `web/src/lib/api/client.ts` — no new API calls
- `web/src/lib/stores/project.svelte.ts` — Material type unchanged

## Module boundaries

```
CatalogFilter.svelte
  ├── props: materials[] (from parent)
  ├── callback: onfilter(filtered[]) (to parent)
  ├── internal: searchQuery, debouncedQuery, activeCategory
  └── imports: Material type from project.svelte.ts

catalog/+page.svelte
  ├── owns: materials[] (fetched from API)
  ├── embeds: CatalogFilter (passes materials, receives filtered)
  ├── renders: table from filtered list
  └── imports: formatUnit, formatPrice from utils/format

MaterialPicker.svelte
  └── imports: formatUnit, formatPrice from utils/format (replaces local copies)
```

## Ordering constraints

1. Create `utils/format.ts` first (no dependencies)
2. Create `CatalogFilter.svelte` (depends on format utils and Material type)
3. Modify `catalog/+page.svelte` (depends on CatalogFilter)
4. Modify `MaterialPicker.svelte` (independent of 2-3, just import swap)
5. Update mock data (independent)
6. Update scenario test (independent of frontend changes)
