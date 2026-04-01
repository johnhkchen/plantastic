---
id: S-028
epic: E-012
title: Visual Consistency & Design Tokens
status: open
priority: low
depends_on: [S-026]
tickets: [T-028-01]
---

## Goal

Establish a design token system (CSS custom properties) and apply it consistently across all pages. Colors, spacing, typography, and border radii should come from tokens, not ad-hoc values. This is the foundation for ★★★☆☆ polish.

## Acceptance Criteria

- Design tokens defined in a single CSS file (`tokens.css` or similar)
- All existing components migrated to use tokens
- Consistent form patterns across create/edit flows (material, project)
- Consistent card/list patterns across dashboard, catalog, zones
- No hardcoded color values outside the token file
