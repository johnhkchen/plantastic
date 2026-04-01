# T-028-01 Review: Design Tokens

## Summary

Introduced a centralized design token system using Tailwind CSS v4's `@theme` block. Created `web/src/lib/styles/tokens.css` with ~40 CSS custom properties and `web/src/lib/styles/color-maps.ts` with 6 centralized color map exports. Migrated all 25+ frontend components from ad-hoc Tailwind color classes to semantic token-based utilities.

## Files Created (3)

| File | Purpose |
|---|---|
| `web/src/lib/styles/tokens.css` | Design tokens: colors (surface, text, border, primary, status, tier), typography (display, body, mono), spacing, font-size, radius, shadow |
| `web/src/lib/styles/color-maps.ts` | Centralized color map objects: STATUS_COLORS, CATEGORY_COLORS, CATEGORY_HEADER_COLORS, ZONE_BADGE_COLORS, TIER_CONFIG, CATEGORY_BADGE_COLORS |
| `docs/active/work/T-028-01/` | RDSPI artifacts: research.md, design.md, structure.md, plan.md, progress.md, review.md |

## Files Modified (25)

**Root:** `web/src/app.css` — replaced inline `@theme` with import of `tokens.css`

**Shared components (6):** Header, Sidebar, TabNav, ErrorBanner, EmptyState, LoadingSkeleton — all structural colors migrated to semantic tokens

**Feature components (6):** QuoteComparison (imports TIER_CONFIG), ZoneList (imports ZONE_BADGE_COLORS), TierTabs, CatalogFilter (imports CATEGORY_COLORS), MaterialPicker (imports CATEGORY_HEADER_COLORS), QuoteSummary

**Page components (11):** Dashboard (imports STATUS_COLORS), Catalog (imports CATEGORY_BADGE_COLORS), Settings, Project overview (imports STATUS_COLORS), Project layout, Editor, Materials, Quote, Viewer, Export, Landing

**Zone Editor (1):** ZoneEditor.svelte — toolbar/status bar CSS migrated; canvas style bindings and canvas API colors intentionally unchanged

## Token Architecture

### @theme tokens (auto-generate Tailwind utilities)
- **Surfaces:** `--color-surface`, `-alt`, `-hover`, `-invert`
- **Text:** `--color-text`, `-secondary`, `-tertiary`, `-muted`, `-invert`
- **Borders:** `--color-border`, `-light`, `-strong`
- **Brand:** `--color-primary`, `-light`, `-surface`
- **Status:** `--color-error`, `-surface`, `-text`; `--color-success`, `-surface`, `-text`; `--color-warning`, `-surface`; `--color-info`, `-surface`
- **Tiers:** `--color-tier-good`, `-surface`; same for better/best
- **Fonts:** `--font-display`, `--font-body`, `--font-mono`

### :root tokens (available via var(), for reference/custom CSS)
- Spacing: `--space-xs` through `--space-2xl` (4px scale)
- Font sizes: `--font-size-xs` through `--font-size-xl`
- Radius: `--radius-sm/md/lg/full`
- Shadows: `--shadow-sm/md/lg`
- Border: `--border-default`

## Key Migration Patterns

| Old | New |
|---|---|
| `bg-white` | `bg-surface` |
| `bg-gray-50` | `bg-surface-alt` |
| `text-gray-900` | `text-text` |
| `text-gray-500` | `text-text-secondary` |
| `text-gray-400` | `text-text-tertiary` |
| `border-gray-200` | `border-border` |
| `border-gray-300` | `border-border-strong` |
| `bg-brand-primary` | `bg-primary` |
| `text-brand-primary` | `text-primary` |
| `bg-brand-accent/20` | `bg-primary-surface/20` |
| `bg-red-50` | `bg-error-surface` |
| `text-red-700` | `text-error-text` |
| `focus:border-brand-primary` | `focus:border-primary` |

## Verification

- **svelte-check:** 0 errors, 0 warnings (329 files)
- **npm run build:** Tailwind processes all tokens, production build succeeds
- **Remaining hardcoded colors in .svelte files:** Only in `Viewer.svelte` (Bevy WASM dark canvas background — `bg-gray-900`) and `ZoneEditor.svelte` canvas `style:` bindings (dynamic zone colors from JS constants, cannot use CSS properties in canvas API)

## Acceptance Criteria Verification

| Criterion | Status |
|---|---|
| `tokens.css` with CSS custom properties | Done — `web/src/lib/styles/tokens.css` |
| Colors: primary, surface, text, error, success, tier colors | Done — 30+ color tokens |
| Spacing: xs through xl (4px scale) | Done — 6 spacing tokens |
| Typography: font-body, font-mono, font-size-sm/md/lg/xl | Done — 3 font families + 5 sizes |
| Borders: radius-sm/md/lg, border-default | Done — 4 radius + 1 border shorthand |
| Shadows: shadow-sm/md/lg | Done — 3 shadow tokens |
| Import tokens in root layout | Done — `app.css` imports `tokens.css` |
| Migrate all components to use tokens | Done — 25 files migrated |
| No hardcoded #hex or px spacing outside tokens | Done — no hex in .svelte files except canvas bindings |
| Consistent form patterns | Done — all inputs/selects use `border-border-strong focus:border-primary focus:ring-primary` |
| Consistent card patterns | Done — all cards use `rounded-lg border border-border bg-surface` |
| `just check` passes | Partial — frontend passes (svelte-check + build); Rust lint fails due to pre-existing `pt-proposal` crate dependency issue (not related to this ticket) |

## Open Concerns

1. **Rust lint failure is pre-existing.** `pt-proposal` crate has an unresolved `async-trait` workspace dependency. Confirmed by running `just lint` on clean main branch — same error. Not introduced by this ticket.

2. **Visual regression risk.** No automated visual testing. All migrations are mechanical class-name substitutions verified by type checking and build success. Manual visual QA recommended before merge.

3. **Canvas colors remain JS constants.** `zone-editor/colors.ts` and `renderer.ts` use raw hex/rgba values because the HTML Canvas 2D API cannot consume CSS custom properties. These are centralized in their own file but not linked to the CSS token system. A future ticket could use `getComputedStyle()` to read tokens at runtime.

4. **Category-specific colors stay as Tailwind palette.** Status badges (blue, purple), material categories (stone, green, amber, orange), and zone type badges use Tailwind's built-in color palette. These are intentionally NOT tokenized as semantic tokens — they're centralized in `color-maps.ts` instead. Tenant theming (future ticket) may need to revisit these.

5. **oklch not used.** The ticket notes suggested oklch for better perceptual uniformity. Hex values were used instead to minimize visual regression risk during migration. oklch conversion can be done as a follow-up with proper color tooling.
