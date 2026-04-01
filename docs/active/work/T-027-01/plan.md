# T-027-01 Plan: Responsive Quote & Catalog

## Step 1: TierTabs tap target fix

**File:** `web/src/lib/components/assignment/TierTabs.svelte`
- Add `min-h-[44px] flex items-center justify-center` to button elements
- Verify: buttons render at ≥44px height

## Step 2: QuoteComparison responsive layout

**File:** `web/src/lib/components/quote/QuoteComparison.svelte`

### 2a: Add imports and state
- Import TierTabs
- Add `activeTier` state variable

### 2b: Refactor loading skeleton
- Change `grid-cols-3` to `grid-cols-1 lg:grid-cols-3`
- Show 1 skeleton on mobile, 3 on desktop

### 2c: Restructure main content into three responsive layouts
- Extract the single-tier card rendering into a shared pattern (or inline 3 times)
- Desktop block: `hidden lg:grid lg:grid-cols-3 lg:gap-4` — all 3 tiers
- Tablet block: `hidden md:flex lg:hidden overflow-x-auto snap-x snap-mandatory gap-4` — all 3 cards, scroll
- Mobile block: `md:hidden` — TierTabs + single active tier card

### 2d: Verify no horizontal overflow
- Cards should never exceed container width at any breakpoint

## Step 3: Catalog page card grid

**File:** `web/src/routes/(app)/catalog/+page.svelte`

### 3a: Replace table with card grid
- Remove `<table>` block
- Add responsive card grid: `grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4`
- Each card: name, category badge, price/unit, edit/delete buttons

### 3b: Ensure tap targets
- Action buttons: `min-h-[44px] min-w-[44px]`

## Step 4: Verify

- Run `just check` (format + lint + test + scenarios)
- Manual verification checklist (for browser dev tools):
  - QuoteComparison at 1440px: 3 columns side by side
  - QuoteComparison at 900px: horizontal scroll with snap
  - QuoteComparison at 375px: tab switcher, one card at a time
  - Catalog at 1440px: 3-column card grid
  - Catalog at 900px: 2-column card grid
  - Catalog at 375px: single column
  - No horizontal overflow at any width
  - All buttons ≥44px tap target

## Testing Strategy

This is a pure CSS/layout change with one small JS addition (activeTier state). No new unit tests needed — this is visual/layout behavior best verified by browser inspection. The existing `just check` pipeline (format, lint, test, scenarios) validates no regressions.

The scenario harness doesn't cover frontend layout — this ticket advances UX quality, not a measurable scenario metric.
