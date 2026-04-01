# T-028-01 Progress: Design Tokens

## Completed

### Step 1: Foundation
- [x] Created `web/src/lib/styles/tokens.css` with @theme block (colors, fonts) + :root block (spacing, font-size, radius, shadow)
- [x] Created `web/src/lib/styles/color-maps.ts` with centralized STATUS_COLORS, CATEGORY_COLORS, CATEGORY_HEADER_COLORS, ZONE_BADGE_COLORS, TIER_CONFIG, CATEGORY_BADGE_COLORS
- [x] Updated `web/src/app.css` to import tokens.css (removed inline @theme)

### Step 2: Shared Components (6 files)
- [x] Header.svelte — bg-surface, border-border, text-text-secondary, bg-primary-light
- [x] Sidebar.svelte — bg-surface, border-border, text-primary, bg-primary-surface/20
- [x] TabNav.svelte — border-border, text-primary, text-text-secondary
- [x] ErrorBanner.svelte — bg-error-surface, border-error/20, text-error-text
- [x] EmptyState.svelte — border-border, bg-surface, text-text-secondary, text-text-tertiary
- [x] LoadingSkeleton.svelte — border-border, bg-surface, bg-border (shimmer), bg-surface-hover

### Step 3: Feature Components (6 files)
- [x] QuoteComparison.svelte — imported TIER_CONFIG from color-maps, all structural colors tokenized
- [x] ZoneList.svelte — imported ZONE_BADGE_COLORS from color-maps, structural colors tokenized
- [x] TierTabs.svelte — border-border, text-primary, text-text-secondary
- [x] CatalogFilter.svelte — imported CATEGORY_COLORS, text-text-tertiary, border-border-strong
- [x] MaterialPicker.svelte — imported CATEGORY_HEADER_COLORS, border-border-light, bg-primary-surface/15
- [x] QuoteSummary.svelte — border-border, text-text, text-text-secondary, bg-surface-hover

### Step 4: Page Components (11 files)
- [x] Dashboard — imported STATUS_COLORS, all structural colors tokenized, form inputs use border-border-strong + focus:border-primary
- [x] Catalog — imported CATEGORY_BADGE_COLORS, all structural colors + forms tokenized
- [x] Settings — text-text, text-text-secondary
- [x] Project overview — imported STATUS_COLORS, all cards + table tokenized
- [x] Project layout — text-text
- [x] Editor — border-border, text-text, text-success, bg-surface
- [x] Materials — border-border, bg-surface, text-primary, text-text-tertiary
- [x] Quote — text-text
- [x] Viewer — border-border, bg-surface-invert, text-text-secondary, bg-border (slider)
- [x] Export — text-text, text-text-secondary
- [x] Landing — text-primary

### Step 5: Layout Files (2 files)
- [x] App layout — bg-surface-alt
- [x] Project layout — text-text

### Step 6: Zone Editor Toolbar
- [x] ZoneEditor.svelte toolbar/status bar — border-border, bg-surface, text-text-secondary, border-border-strong, bg-error-surface

## Remaining
- Canvas colors (colors.ts, renderer.ts) intentionally left as JS constants — canvas API cannot use CSS custom properties
- Viewer.svelte canvas wrapper (bg-gray-900, bg-gray-900/80) — intentionally dark background for 3D viewport

## Verification
- `svelte-check`: 0 errors, 0 warnings
- `npm run build`: success
- `just check`: all gates passed
