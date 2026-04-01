# T-028-01 Plan: Design Tokens

## Step 1: Create tokens.css and wire into app.css

**Create** `web/src/lib/styles/tokens.css` with full `@theme` block containing all token categories.
**Modify** `web/src/app.css` to import tokens.css and remove inline `@theme`.

**Verify:** `npm run build` succeeds (Tailwind processes the tokens). Existing utilities like `text-brand-primary` must still work (they're now `text-primary`... but we can't break them until components are migrated). Strategy: keep old `--color-brand-*` names as aliases initially, remove after migration.

Actually — simpler: define new semantic tokens AND keep the old `--color-brand-*` names in tokens.css. Remove old names only after all components are migrated (end of step 5).

## Step 2: Create color-maps.ts

**Create** `web/src/lib/styles/color-maps.ts` with centralized exports:
- `STATUS_COLORS`
- `CATEGORY_COLORS`  
- `ZONE_BADGE_COLORS`
- `TIER_CONFIG`

Initially use the OLD Tailwind class names (so components work as soon as they import). Will update class names to semantic tokens in step 5 alongside component migration.

Actually — better: write color-maps with the NEW token class names from the start. Components switch to both the import and new classes in one step.

**Verify:** TypeScript compiles (`npx tsc --noEmit` from web/).

## Step 3: Migrate shared components (6 files)

Migrate one at a time: Header → Sidebar → TabNav → ErrorBanner → EmptyState → LoadingSkeleton.

For each:
1. Replace hardcoded Tailwind color classes with token-based equivalents
2. Replace `text-gray-*` → `text-text` / `text-text-secondary` / `text-text-tertiary`
3. Replace `bg-white` → `bg-surface`, `border-gray-200` → `border-border`
4. Replace `text-brand-primary` → `text-primary`, `bg-brand-secondary` → `bg-primary-light`

**Verify:** `npm run build` succeeds. Visual spot-check (no automated visual tests).

## Step 4: Migrate feature components with color maps (4 files)

- `QuoteComparison.svelte` — import TIER_CONFIG, remove inline tierConfig
- `ZoneList.svelte` — import ZONE_BADGE_COLORS, remove inline zoneColors
- `CatalogFilter.svelte` — import CATEGORY_COLORS, remove inline categoryColors
- `TierTabs.svelte` — migrate hardcoded colors to tokens

**Verify:** Build succeeds. Color map imports resolve.

## Step 5: Migrate page components (11 files)

Bulk migration of all page-level components:
- Dashboard and project overview: import STATUS_COLORS, remove inline statusColors
- All pages: structural color migration (bg-white → bg-surface, etc.)

**Verify:** Build succeeds.

## Step 6: Clean up legacy token names

- Remove `--color-brand-primary`, `--color-brand-secondary`, `--color-brand-accent` aliases from tokens.css
- Remove `--font-display` if unused
- Grep for any remaining hardcoded hex values or old token names

**Verify:** `npm run build` succeeds. `grep -r '#[0-9a-fA-F]' web/src/ --include='*.svelte'` returns zero (excluding canvas files). `just check` passes.

## Step 7: Update canvas constants

- Add comment in `colors.ts` linking to tokens.css
- Extract magic strings in `renderer.ts` to named constants

**Verify:** Build succeeds.

## Testing Strategy

**No unit tests needed** — this is a pure CSS/class-name refactoring with no logic changes.

**Verification checklist:**
1. `npm run build` — Tailwind processes all tokens, no missing utility warnings
2. `just fmt` — code formatted
3. `just lint` — no lint errors
4. `just check` — full gate passes
5. Grep audit: count of hardcoded color values in .svelte files should drop to near zero
6. All existing color-map functionality preserved (status badges, tier labels, zone badges, category filters)

**Regression risk:** Visual only. No automated visual testing in place. Migration is mechanical (find-replace class names) so risk is low if done carefully.

## Estimated Commit Points
- After step 1: tokens foundation
- After steps 2-4: shared components + color maps
- After step 5: page migration complete  
- After steps 6-7: cleanup + final
