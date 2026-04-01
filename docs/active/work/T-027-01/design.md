# T-027-01 Design: Responsive Quote & Catalog

## QuoteComparison — Three Approaches

### Option A: Tailwind responsive grid + TierTabs on mobile

- Desktop (≥1024px): keep existing 3-column grid (`lg:grid-cols-3`)
- Tablet (768–1023px): horizontal scroll with snap (`md:flex md:overflow-x-auto md:snap-x`)
- Mobile (<768px): show TierTabs, render one tier card at a time

**Pros:** Clean mobile UX with familiar tab pattern. Reuses existing TierTabs.
**Cons:** Requires small JS state for active tier on mobile. Mixing CSS visibility with JS state.

### Option B: Pure CSS horizontal scroll at all sub-desktop sizes

- Desktop: 3-column grid
- Below 1024px: horizontal scroll container with scroll-snap

**Pros:** Zero JS. Simple implementation.
**Cons:** Poor mobile UX — user must scroll horizontally to compare. Hard to see all tiers. Scroll-snap UX is mediocre on small phones.

### Option C: CSS-only stacked cards below 1024px

- Desktop: 3-column grid
- Below 1024px: stack all three cards vertically

**Pros:** Zero JS, simple CSS.
**Cons:** Very long scroll on mobile. Hard to compare tiers. Bad UX.

### Decision: Option A (Tailwind responsive + TierTabs on mobile)

The ticket AC explicitly calls for "stacked cards, one tier at a time with tab switcher" on mobile and "scrollable tabs or horizontal scroll with snap" on tablet. Option A maps directly to the AC. The TierTabs component already exists and matches the design language.

The "small JS" concern is minimal — it's a single `$state` variable for `activeTier`, and the TierTabs component already handles the interaction.

**Tablet approach:** Horizontal scroll with snap. The 3 cards get `min-w-[300px]` and the container gets `overflow-x-auto snap-x snap-mandatory`. Each card gets `snap-center`. This matches the AC's "scrollable tabs or horizontal scroll with snap."

## Catalog — Two Approaches

### Option A: Responsive table with card fallback on mobile

- Desktop (≥1024px): 3-column grid of cards (not table)
- Tablet: 2-column grid
- Mobile: single column

**Problem:** The current catalog is a table, and the AC says "3-column grid" at desktop. This implies converting from table to card-based grid.

### Option B: Keep table on desktop, card list on mobile

- Desktop: keep table as-is (it works well for tabular data)
- Tablet: keep table with horizontal scroll
- Mobile: switch to card layout

**Problem:** AC says "3-column grid" at ≥1024. The current table isn't a grid. But reading more carefully — the AC may be describing the desired responsive behavior of the catalog items themselves, not requiring a grid redesign. The table works fine on desktop.

### Decision: Convert catalog to responsive card grid

The AC is clear: "≥1024px: 3-column grid, 768–1023px: 2-column grid, <768px: single column list." This means replacing the table with a card-based layout that uses responsive Tailwind grid classes.

Card layout per material:
- Name (bold), category badge, unit, price
- Edit/Delete actions
- Compact enough for 3 columns at desktop

Grid classes: `grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4`

## Tap Target Sizing

Both components need ≥44px tap targets on all interactive elements:
- TierTabs buttons: currently `px-4 py-2` — need `min-h-[44px]` or `py-3`
- Catalog card buttons: ensure `min-h-[44px] min-w-[44px]`
- Quote zone rows: not interactive, no change needed

## Summary of Changes

1. **QuoteComparison.svelte**: Add mobile tier tabs + responsive grid
2. **catalog/+page.svelte**: Replace table with responsive card grid
3. **TierTabs.svelte**: Add `min-h-[44px]` for tap target compliance
