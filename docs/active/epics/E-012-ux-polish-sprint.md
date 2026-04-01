---
id: E-012
title: UX Polish Sprint
status: open
priority: medium
sprint: 2
depends_on: [E-010]
---

## Context

Every passing scenario is at ★☆☆☆☆ polish. The SvelteKit frontend works but has no loading states, no error handling, no empty states, inconsistent styling, and no responsive layout. This is the gap between "it works in a demo where I drive" and "a landscaper can use it without hand-holding."

This epic pushes polish from ★☆☆☆☆ to ★★☆☆☆ or ★★★☆☆ across all frontend-facing scenarios. The target is basic professionalism: loading indicators, error messages, empty state prompts, and layouts that don't break on tablet.

## Areas

| Area | Scenarios Affected | Current Polish |
|------|-------------------|----------------|
| Dashboard + Project | S.2.1, S.1.2 | ★☆☆☆☆ |
| Catalog | S.2.2 | ★☆☆☆☆ |
| Quote Flow | S.3.1, S.3.2 | ★☆☆☆☆ |
| 3D Viewer | S.2.4 | ★☆☆☆☆ |

## Stories

- S-026: Core UX Polish (loading, errors, empty states)
- S-027: Responsive & Tablet Layout
- S-028: Visual Consistency & Design Tokens

## Success Criteria

- All frontend-facing scenarios at ★★☆☆☆ polish minimum
- Loading states on every async page/component
- Error boundaries with user-friendly messages
- Empty states with CTAs on dashboard, catalog, zones, quotes
- Quote comparison readable on iPad-width viewport
- `just check` passes
