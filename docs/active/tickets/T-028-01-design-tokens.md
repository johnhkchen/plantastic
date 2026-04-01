---
id: T-028-01
story: S-028
title: design-tokens
type: task
status: open
priority: low
phase: open
depends_on: [T-026-01]
---

## Context

Frontend components use ad-hoc colors, spacing, and typography values. No shared design system means inconsistent look across pages. Design tokens are the foundation for ★★★☆☆ polish.

## Acceptance Criteria

- Create `web/src/lib/styles/tokens.css` with CSS custom properties:
  - Colors: `--color-primary`, `--color-surface`, `--color-text`, `--color-error`, `--color-success`, tier colors (good/better/best)
  - Spacing: `--space-xs` through `--space-xl` (4px scale)
  - Typography: `--font-body`, `--font-mono`, `--font-size-sm/md/lg/xl`
  - Borders: `--radius-sm/md/lg`, `--border-default`
  - Shadows: `--shadow-sm/md/lg`
- Import tokens in root layout
- Migrate all existing components to use tokens (no hardcoded `#hex` or `px` spacing outside tokens)
- Consistent form patterns: input fields, buttons, labels, validation messages
- Consistent card patterns: project card, material card, zone card
- `just check` passes

## Implementation Notes

- Start with a neutral palette — tenant branding (★★★★☆) comes later
- Use `oklch()` for color tokens if browser support allows (better perceptual uniformity)
- Don't introduce a CSS framework — plain CSS custom properties are sufficient
- Count of hardcoded color values should drop to near zero
