# T-028-01 Design: Design Tokens

## Decision: Tailwind v4 @theme tokens in dedicated CSS file

### Approach

Create `web/src/lib/styles/tokens.css` with all design tokens as CSS custom properties inside a Tailwind v4 `@theme` block. Import this file from `app.css` (replacing the inline `@theme`). Migrate all components to use the generated utility classes.

For canvas/JS colors, export a parallel `tokens.ts` that reads CSS custom properties at runtime via `getComputedStyle`, with static fallbacks for SSR/canvas contexts.

### Options Considered

**Option A: Expand inline `@theme` in app.css**
- Pros: No new files, simple
- Cons: `app.css` becomes large; mixes Tailwind import with token definitions; harder to find tokens
- Rejected: Doesn't meet AC requirement for `tokens.css` in `lib/styles/`

**Option B: Separate `tokens.css` imported into `app.css` (chosen)**
- Pros: Clean separation; `app.css` stays minimal; tokens file is the single source of truth; meets AC path requirement; Tailwind v4 `@theme` in imported files works natively
- Cons: One extra import
- Selected: Best balance of organization and simplicity

**Option C: CSS Modules or Sass variables**
- Rejected: Adds build complexity; Tailwind v4 `@theme` already does this natively; ticket explicitly says "plain CSS custom properties"

**Option D: JS-first tokens (Style Dictionary, etc.)**
- Rejected: Overkill for current needs; adds build dependency; CSS custom properties are sufficient

### Token Architecture

#### Color Tokens (oklch)

Use oklch for all custom colors. Map to semantic names, not hue names.

```
/* Surfaces */
--color-surface: oklch(1 0 0);              /* white — page/card background */
--color-surface-alt: oklch(0.985 0 0);      /* gray-50 equiv — alternate bg */
--color-surface-hover: oklch(0.97 0 0);     /* gray-100 equiv */
--color-surface-invert: oklch(0.21 0.006 285); /* gray-900 equiv — dark bg */

/* Text */
--color-text: oklch(0.21 0.006 285);        /* gray-900 — primary text */
--color-text-secondary: oklch(0.55 0.014 285); /* gray-500 — secondary */
--color-text-tertiary: oklch(0.65 0.014 285);  /* gray-400 — muted */
--color-text-invert: oklch(1 0 0);          /* white text on dark bg */

/* Borders */
--color-border: oklch(0.93 0.003 285);      /* gray-200 — default border */
--color-border-strong: oklch(0.87 0.005 285); /* gray-300 */

/* Brand (keep existing hex, convert to oklch) */
--color-primary: oklch(0.49 0.104 163);     /* #2d6a4f */
--color-primary-light: oklch(0.58 0.104 163); /* #40916c */
--color-primary-surface: oklch(0.83 0.08 163); /* #95d5b2 */

/* Status */
--color-error: oklch(0.58 0.2 25);          /* red-600 */
--color-error-surface: oklch(0.97 0.01 25); /* red-50 */
--color-success: oklch(0.55 0.16 145);      /* green-600 */
--color-success-surface: oklch(0.97 0.01 145); /* green-50 */
--color-warning: oklch(0.7 0.15 75);        /* amber-500 */
--color-warning-surface: oklch(0.97 0.02 80); /* amber-50 */
--color-info: oklch(0.55 0.15 250);         /* blue-600 */
--color-info-surface: oklch(0.97 0.01 250); /* blue-50 */

/* Tiers (Good/Better/Best) */
--color-tier-good: var(--color-text-secondary);
--color-tier-good-surface: var(--color-surface-alt);
--color-tier-better: var(--color-primary);
--color-tier-better-surface: oklch(0.97 0.02 163);
--color-tier-best: oklch(0.55 0.12 70);     /* amber-700 */
--color-tier-best-surface: oklch(0.97 0.02 80);
```

#### Spacing Tokens

Map to Tailwind's default scale but give semantic aliases where useful:

```
--space-xs: 0.25rem;   /* 4px — tight gaps */
--space-sm: 0.5rem;    /* 8px — compact spacing */
--space-md: 1rem;      /* 16px — default spacing */
--space-lg: 1.5rem;    /* 24px — section spacing */
--space-xl: 2rem;      /* 32px — large gaps */
--space-2xl: 3rem;     /* 48px — page margins */
```

#### Typography Tokens

```
--font-body: 'Inter', system-ui, sans-serif;
--font-mono: 'JetBrains Mono', ui-monospace, monospace;
--font-size-xs: 0.75rem;
--font-size-sm: 0.875rem;
--font-size-md: 1rem;
--font-size-lg: 1.125rem;
--font-size-xl: 1.25rem;
--font-size-2xl: 1.5rem;
```

Note: `--font-display` is dropped — same value as `--font-body`. One font family simplifies.

#### Border & Shadow Tokens

```
--radius-sm: 0.375rem;  /* 6px — buttons, inputs */
--radius-md: 0.5rem;    /* 8px — cards */
--radius-lg: 0.75rem;   /* 12px — modals */
--radius-full: 9999px;  /* pills, avatars */

--border-default: 1px solid var(--color-border);

--shadow-sm: 0 1px 2px 0 oklch(0 0 0 / 0.05);
--shadow-md: 0 4px 6px -1px oklch(0 0 0 / 0.1);
--shadow-lg: 0 10px 15px -3px oklch(0 0 0 / 0.1);
```

### Migration Strategy

**Phase 1: Define tokens, import in root.** No component changes yet — just establish the file.

**Phase 2: Migrate shared components first** (Header, Sidebar, TabNav, ErrorBanner, EmptyState, LoadingSkeleton). These set the visual language.

**Phase 3: Migrate page components.** Replace hardcoded Tailwind color classes with semantic token classes.

**Phase 4: Centralize color maps.** Move status/category/zone/tier color definitions to a shared `colorMaps.ts` that references token-based classes.

**Phase 5: Canvas colors.** Update `colors.ts` and `renderer.ts` to use centralized constants that match the CSS tokens.

### What This Does NOT Do
- Tenant branding / theme switching (future ticket)
- Dark mode (not in scope)
- Responsive typography scales (not needed yet)
- CSS framework introduction (explicitly excluded by ticket)

### Key Decision: Keep Tailwind Utilities
We are NOT replacing Tailwind utility classes with raw `var(--token)` in style attributes. Instead, `@theme` tokens generate Tailwind utilities (e.g., `--color-surface` becomes `bg-surface`, `text-surface`). Components use these Tailwind classes. This preserves the existing DX while centralizing values.

Where Tailwind's built-in palette colors (gray-200, etc.) are used for structural purposes (borders, backgrounds), we alias them to semantic tokens. Components then use `border-border` instead of `border-gray-200`.
