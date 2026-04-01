---
id: T-027-01
story: S-027
title: responsive-quote-catalog
type: task
status: open
priority: medium
phase: done
depends_on: [T-026-01]
---

## Context

The quote comparison page uses a 3-column CSS grid that overflows on viewports < 1024px. The catalog page has no responsive breakpoints. Both break on iPad.

## Acceptance Criteria

- Quote comparison:
  - ≥1024px: 3-column side-by-side (current)
  - 768–1023px: scrollable tabs or horizontal scroll with snap
  - <768px: stacked cards, one tier at a time with tab switcher
- Catalog:
  - ≥1024px: 3-column grid
  - 768–1023px: 2-column grid
  - <768px: single column list
- No horizontal overflow at any breakpoint
- Touch-friendly: all interactive elements ≥ 44px tap target
- `just check` passes

## Implementation Notes

- Use CSS media queries, not JS resize observers
- `@media (max-width: 768px)` for mobile, `@media (max-width: 1023px)` for tablet
- Quote tabs on mobile: reuse existing TierTabs component or add a simple tab bar
- Test in browser dev tools at iPad (768×1024) and iPhone SE (375×667)
