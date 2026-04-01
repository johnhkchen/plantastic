---
id: T-012-02
story: S-012
title: catalog-search-filter
type: task
status: open
priority: medium
phase: done
depends_on: [T-012-01]
---

## Context

Make the catalog usable at scale. A landscaper's catalog might have 50-200 materials. Search and category filtering lets them find what they need quickly — important both in the catalog page and later in the material picker during zone assignment.

## Acceptance Criteria

- Search input: filters materials by name (client-side for V1, debounced)
- Category filter tabs or dropdown: hardscape, softscape, edging, fill, all
- Combined: search within a category
- Material count display per category
- Reusable as a component (will be embedded in the material assignment UI later)
- S.2.2 scenario passes at ★★ (catalog with search/filter reachable via UI)
- Claim milestone: "pt-materials: catalog model + tenant layering"
