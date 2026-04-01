# T-027-01 Structure: Responsive Quote & Catalog

## Files Modified

### 1. `web/src/lib/components/quote/QuoteComparison.svelte`

**Script changes:**
- Import `TierTabs` from `$lib/components/assignment/TierTabs.svelte`
- Add `activeTier` state: `let activeTier = $state<string>('good')`

**Template changes — loading state:**
- Change `grid grid-cols-3` → responsive: `grid grid-cols-1 lg:grid-cols-3`
- On mobile: show single skeleton card. On desktop: show 3.

**Template changes — main content:**
- Wrap in outer container with two rendering modes:

  **Desktop (≥1024px):** `hidden lg:grid lg:grid-cols-3 lg:gap-4`
  - Existing 3-column grid, unchanged

  **Tablet (768–1023px):** `hidden md:flex md:overflow-x-auto md:snap-x md:snap-mandatory md:gap-4 lg:hidden`
  - Each card: `min-w-[300px] snap-center flex-shrink-0`
  - Container: `-mx-4 px-4` for edge-to-edge scroll with padding

  **Mobile (<768px):** `md:hidden`
  - TierTabs component at top
  - Single card for `activeTier` only

**Note:** Using Tailwind's `hidden`/`block`/`flex` responsive utilities to show/hide layouts is pure CSS (compiles to `@media` queries). The only JS is `activeTier` state, which is only read on mobile.

### 2. `web/src/routes/(app)/catalog/+page.svelte`

**Template changes — material list:**
- Replace `<table>` block (lines 164–224) with card grid:
  ```
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
    {#each filteredMaterials as mat (mat.id)}
      <div class="rounded-lg border border-gray-200 bg-white p-4">
        <!-- name, category badge, unit × price, actions -->
      </div>
    {/each}
  </div>
  ```
- Each card contains:
  - Header row: material name + category badge
  - Price row: formatted price/unit
  - Footer: Edit and Delete buttons with `min-h-[44px]`

**No script changes** — all data flow remains identical.

### 3. `web/src/lib/components/assignment/TierTabs.svelte`

**Minor change:**
- Add `min-h-[44px]` to tab buttons for touch target compliance
- Add `flex items-center justify-center` for vertical centering with taller buttons

## Files NOT Changed

- `web/src/routes/(app)/project/[id]/quote/+page.svelte` — wrapper only, no layout concerns
- `web/src/lib/components/catalog/CatalogFilter.svelte` — already within `max-w-4xl`, works fine
- Modal in catalog page — already `max-w-md`, mobile-friendly

## Architecture Notes

- No new components created. Reuse TierTabs.
- No new dependencies.
- All responsive behavior via Tailwind responsive prefixes (CSS media queries).
- Mobile tier switching is the only JS addition (one `$state` variable).
