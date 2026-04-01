# T-028-01 Research: Design Tokens

## Current State

### Styling Architecture
- **Framework:** Tailwind CSS v4.2.2 via `@tailwindcss/vite`
- **Global styles:** `web/src/app.css` — imports Tailwind, defines a `@theme` block
- **Root layout:** `web/src/routes/+layout.svelte` — imports `app.css`, no other global styling
- **No `web/src/lib/styles/` directory exists** — ticket AC calls for `tokens.css` here

### Existing Design Tokens (5 total)
In `app.css` `@theme` block:
- `--color-brand-primary: #2d6a4f` (dark green)
- `--color-brand-secondary: #40916c` (medium green)
- `--color-brand-accent: #95d5b2` (light green)
- `--font-display: 'Inter', system-ui, sans-serif`
- `--font-body: 'Inter', system-ui, sans-serif`

Tailwind v4 `@theme` makes these available as utility classes (`text-brand-primary`, `font-display`, etc.).

### Component Inventory (28 files)
**Layouts (3):** root, app, project  
**Pages (11):** landing, dashboard, catalog, settings, project overview, editor, materials, quote, viewer, export, client share  
**Shared components (6):** Header, Sidebar, TabNav, ErrorBanner, EmptyState, LoadingSkeleton  
**Feature components (8):** ZoneEditor, TierTabs, MaterialPicker, ZoneList, QuoteSummary, QuoteComparison, CatalogFilter, Viewer

### Hardcoded Color Hotspots

**1. Status color maps — duplicated in 2 files**
- `dashboard/+page.svelte` lines 74-79
- `project/[id]/+page.svelte` lines 48-53
- Values: `draft→gray, quoted→blue, approved→green, complete→purple`

**2. Category color maps — CatalogFilter.svelte lines 11-28**
- `hardscape→stone, softscape→green, edging→amber, fill→orange`
- Each has active/inactive variant (2 class strings each)

**3. Zone type color maps — 2 separate systems**
- Canvas rendering: `zone-editor/colors.ts` — hex+rgba for `<canvas>` API
- UI badges: `assignment/ZoneList.svelte` lines 18-25 — Tailwind classes
- These are intentionally different (canvas vs CSS) but share no common source

**4. Quote tier colors — QuoteComparison.svelte lines 15-19**
- `good→gray, better→brand-primary, best→amber`

**5. Canvas hardcoded colors — renderer.ts**
- Grid: `#e5e7eb`, handles: `#ffffff`/`#374151`
- Font: `'12px Inter, system-ui, sans-serif'` hardcoded in canvas context

### Hardcoded Spacing Patterns
- Card padding: inconsistent mix of `p-4`, `p-5`, `p-6`
- Page padding: `px-6` in Header, `px-4` in Sidebar items
- Gaps: `gap-1` through `gap-4` with no clear scale
- One calc expression: `h-[calc(100vh-12rem)]` in materials page

### Hardcoded Typography
- Font sizes: `text-xs`, `text-sm`, `text-lg`, `text-2xl`, `text-3xl`, `text-4xl`
- Arbitrary sizes: `text-[10px]`, `text-[11px]` in zone editor
- Weights: `font-medium`, `font-semibold`, `font-bold`
- Tracking: `uppercase tracking-wider`, `uppercase tracking-wide`

### Hardcoded Borders/Shadows
- Radius: `rounded-md`, `rounded-lg`, `rounded-full` — 3 values used consistently
- Shadows: `shadow-sm` (17 uses), `shadow-xl` (2 uses — modals)
- Border pattern: `border border-gray-200` is the universal card/divider color

### Constraints & Considerations

1. **Tailwind v4 `@theme` is the extension point.** CSS custom properties defined in `@theme` auto-generate utility classes. This is the correct place for tokens — no separate build step needed.

2. **Canvas API cannot use CSS custom properties directly.** Zone editor `colors.ts` and `renderer.ts` use raw color strings for `<canvas>` 2D context. These must remain JS constants but can import from a shared source.

3. **Ticket says `tokens.css`** but the existing `@theme` block in `app.css` is already the Tailwind v4 way. Creating a separate `tokens.css` and importing it into `app.css` is cleaner than bloating `app.css`.

4. **oklch() suggestion in ticket notes.** Browser support is good (95%+ on caniuse). oklch gives perceptually uniform lightness scales. Tailwind v4 supports it natively.

5. **Tenant branding comes later (ticket notes).** Token naming should be semantic (e.g., `--color-surface`) not tied to specific hues, so tenant theming can swap values without renaming classes.

6. **"No hardcoded #hex or px spacing outside tokens" is the AC.** For Tailwind, this means all color/spacing utilities should map to tokens defined in `@theme`. Standard Tailwind palette colors (`gray-500`) are already token-based in Tailwind v4, but the AC implies we should alias them to semantic names.

7. **Color maps in JS objects** (status, category, zone, tier) are the hardest to tokenize. These use Tailwind class strings, not raw values. The migration path is to replace bare Tailwind colors with semantic token-based classes.

### Files That Will Change
- `web/src/app.css` — expand `@theme` block or import tokens
- `web/src/lib/styles/tokens.css` — new file (per AC)
- `web/src/lib/components/zone-editor/colors.ts` — import from shared source
- `web/src/lib/components/zone-editor/renderer.ts` — use token constants
- All 6 shared components — migrate to token classes
- All 11 page components — migrate to token classes
- Color map objects in QuoteComparison, ZoneList, CatalogFilter, dashboard, project overview

### Risks
- **Large surface area:** 28 files need class name changes. Risk of visual regressions without visual testing.
- **Canvas colors stay JS:** Cannot fully eliminate hardcoded hex in canvas code, only centralize it.
- **Semantic naming decisions:** Choosing the right abstraction level for token names affects future extensibility.
