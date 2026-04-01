# T-027-01 Research: Responsive Quote & Catalog

## Scope

Two pages need responsive breakpoints:
1. **QuoteComparison** component (`web/src/lib/components/quote/QuoteComparison.svelte`)
2. **Catalog** page (`web/src/routes/(app)/catalog/+page.svelte`)

## QuoteComparison Component

**Location:** `web/src/lib/components/quote/QuoteComparison.svelte`
**Used by:** `web/src/routes/(app)/project/[id]/quote/+page.svelte`

### Current Layout

- Loading state: `grid grid-cols-3 gap-4` (line 81) — hardcoded 3 columns
- Main content: `grid grid-cols-3 gap-4` (line 101) — hardcoded 3 columns
- Each tier is a flex column card with header, zone rows, and footer total
- Cards have no min-width constraint — they compress to ~33% viewport width

### Key Data Flow

- Props: `quotes` object with `good`, `better`, `best` keys; `loading` boolean
- `TIERS` array: `['good', 'better', 'best']`
- `TIER_CONFIG`: label, color, bg per tier
- `allZoneIds`: derived from all tiers' line items
- `zoneItemLookup`: zoneId → tier → LineItem

### Existing Tier Tab Component

`web/src/lib/components/assignment/TierTabs.svelte`:
- Simple button bar with `good`/`better`/`best` tabs
- Uses `$bindable()` for `activeTier` prop
- Already styled with brand colors and border-b-2 active indicator
- **Reusable for mobile quote view**

## Catalog Page

**Location:** `web/src/routes/(app)/catalog/+page.svelte`

### Current Layout

- Uses an HTML `<table>` with 5 columns: Name, Category, Unit, Price, Actions
- Table wrapped in `overflow-hidden rounded-lg border`
- No responsive breakpoints at all
- Table will horizontal-overflow on narrow screens
- Existing `CatalogFilter` component sits above the table
- Modal for create/edit is `max-w-md` — already responsive-friendly

### Other Observations

- The codebase uses Tailwind CSS throughout
- `TabNav.svelte` shows `overflow-x-auto` pattern for horizontal scrolling
- No existing CSS media queries or responsive utilities beyond Tailwind defaults
- All button/link elements appear to be standard size — need to verify ≥44px tap targets
- The project uses SvelteKit with Svelte 5 runes (`$state`, `$derived`, `$props`)

## Constraints

- Ticket says: use CSS media queries, not JS resize observers
- Tailwind responsive prefixes (`sm:`, `md:`, `lg:`) compile to media queries — valid approach
- Breakpoints to target:
  - `<768px` → mobile (Tailwind: default, then override at `md:`)
  - `768–1023px` → tablet (Tailwind: `md:` prefix = 768px)
  - `≥1024px` → desktop (Tailwind: `lg:` prefix = 1024px)

## Dependencies

- T-026-01 (loading/error states) — marked as dependency, already implemented per git status
- No runtime JS dependencies needed — pure CSS/Tailwind solution

## Risk Areas

- QuoteComparison mobile: need to add tab state for mobile tier switcher
  - This requires a small amount of reactive state (activeTier) — minimal JS
  - Can reuse TierTabs component directly
- Catalog table → card transformation on mobile is a common pattern
  - Need to hide table, show cards (or use CSS to reshape)
  - Simpler: use Tailwind responsive classes to switch between table and card layout
