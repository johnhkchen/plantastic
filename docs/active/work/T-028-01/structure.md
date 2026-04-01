# T-028-01 Structure: Design Tokens

## New Files

### `web/src/lib/styles/tokens.css`
Single source of truth for all design tokens. Contains one `@theme` block with:
- Color tokens (surfaces, text, borders, brand, status, tiers)
- Spacing tokens (xs through 2xl)
- Typography tokens (font families, sizes)
- Border/radius tokens
- Shadow tokens

Organized in sections with comment headers. ~80 custom properties total.

### `web/src/lib/styles/color-maps.ts`
Centralized color map objects used by multiple components. Exports:
- `STATUS_COLORS: Record<ProjectStatus, string>` — Tailwind class strings for draft/quoted/approved/complete
- `CATEGORY_COLORS: Record<MaterialCategory, { active: string; inactive: string }>` — for catalog filters
- `ZONE_BADGE_COLORS: Record<ZoneType, string>` — Tailwind class strings for zone type badges
- `TIER_CONFIG: Record<TierName, { label: string; color: string; bg: string }>` — for quote comparison

These replace duplicated inline objects across components. All class strings reference token-based utilities.

## Modified Files

### `web/src/app.css`
- Remove inline `@theme` block (5 properties)
- Add `@import './lib/styles/tokens.css';` after Tailwind import
- Keep `@import 'tailwindcss';` as first line

### Shared Components (6 files)

**`web/src/lib/components/Header.svelte`**
- `bg-white` → `bg-surface`
- `border-gray-200` → `border-border`
- `bg-brand-secondary` → `bg-primary-light`
- `text-brand-primary` → `text-primary`

**`web/src/lib/components/Sidebar.svelte`**
- `bg-white` → `bg-surface`
- `border-gray-200` → `border-border`
- `text-gray-600` → `text-text-secondary`
- `bg-brand-accent/20` → `bg-primary-surface/20`
- `text-brand-primary` → `text-primary`

**`web/src/lib/components/TabNav.svelte`**
- `border-gray-200` → `border-border`
- `text-gray-500` → `text-text-secondary`
- `text-brand-primary` → `text-primary`
- `border-brand-primary` → `border-primary`

**`web/src/lib/components/ErrorBanner.svelte`**
- `bg-red-50` → `bg-error-surface`
- `border-red-200` → `border-error/20`
- `text-red-600` → `text-error`

**`web/src/lib/components/EmptyState.svelte`**
- `border-gray-200` → `border-border`
- `bg-white` → `bg-surface`
- `text-gray-500` → `text-text-secondary`
- `text-gray-400` → `text-text-tertiary`

**`web/src/lib/components/LoadingSkeleton.svelte`**
- `border-gray-200` → `border-border`
- `bg-white` → `bg-surface`
- `bg-gray-200` → `bg-surface-hover`
- `bg-gray-100` → `bg-surface-alt`

### Feature Components (4 files with color maps)

**`web/src/lib/components/quote/QuoteComparison.svelte`**
- Remove inline `tierConfig` object
- Import `TIER_CONFIG` from `color-maps.ts`

**`web/src/lib/components/assignment/ZoneList.svelte`**
- Remove inline `zoneColors` object
- Import `ZONE_BADGE_COLORS` from `color-maps.ts`

**`web/src/lib/components/catalog/CatalogFilter.svelte`**
- Remove inline `categoryColors` object
- Import `CATEGORY_COLORS` from `color-maps.ts`

**`web/src/lib/components/assignment/TierTabs.svelte`**
- Migrate hardcoded colors to token-based classes

### Page Components (migrate colors/borders to tokens)

**`web/src/routes/(app)/dashboard/+page.svelte`**
- Remove inline `statusColors` object → import `STATUS_COLORS`
- Migrate structural colors: `bg-white` → `bg-surface`, `border-gray-200` → `border-border`, etc.
- `text-gray-900` → `text-text`, `text-gray-500` → `text-text-secondary`

**`web/src/routes/(app)/project/[id]/+page.svelte`**
- Remove inline `statusColors` → import `STATUS_COLORS`
- Same structural color migration

**`web/src/routes/(app)/catalog/+page.svelte`**
- Structural color migration only

**`web/src/routes/(app)/project/[id]/editor/+page.svelte`**
- Structural color migration

**`web/src/routes/(app)/project/[id]/materials/+page.svelte`**
- Structural color migration

**`web/src/routes/(app)/project/[id]/quote/+page.svelte`**
- Structural color migration

**`web/src/routes/(app)/project/[id]/viewer/+page.svelte`**
- Structural color migration

**`web/src/routes/(app)/project/[id]/export/+page.svelte`**
- Structural color migration

**`web/src/routes/(app)/settings/+page.svelte`**
- Structural color migration

**`web/src/routes/+page.svelte`** (landing)
- Structural color migration

**`web/src/routes/c/[token]/+page.svelte`** (client share)
- Structural color migration

### Canvas/Zone Files (2 files — partial tokenization)

**`web/src/lib/components/zone-editor/colors.ts`**
- Colors stay as JS constants (canvas API needs raw strings)
- Add comment linking to tokens.css for the semantic mapping
- No functional change — these are canvas-specific and intentionally separate

**`web/src/lib/components/zone-editor/renderer.ts`**
- Replace magic strings with named constants from colors.ts
- `GRID_COLOR`, `HANDLE_FILL`, `HANDLE_STROKE` stay as JS but get named exports
- Canvas font string stays hardcoded (canvas API limitation)

## Files NOT Changed
- `web/src/app.html` — no style changes needed
- `web/src/lib/components/viewer/Viewer.svelte` — Bevy WASM canvas, no CSS
- `web/src/lib/components/assignment/MaterialPicker.svelte` — review for changes
- `web/src/lib/components/assignment/QuoteSummary.svelte` — review for changes
- `web/src/lib/components/zone-editor/ZoneEditor.svelte` — canvas dynamic styles stay

## Module Boundaries
- `tokens.css` is imported only by `app.css` — all components access tokens via Tailwind utilities
- `color-maps.ts` is imported by individual components — no circular deps
- Zone editor canvas colors remain a separate system (JS-only, canvas API)

## Ordering
1. Create `tokens.css` + update `app.css` import (foundation)
2. Create `color-maps.ts` (shared dependency)
3. Migrate shared components (establish pattern)
4. Migrate feature components (use color-maps)
5. Migrate page components (bulk work)
6. Update canvas constants (cleanup)
