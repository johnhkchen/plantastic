---
id: T-012-01
story: S-012
title: catalog-crud-page
type: task
status: open
priority: high
phase: done
depends_on: [T-006-01]
---

## Context

The material catalog management page. Landscapers need to add their materials before they can assign them to zones. This is both a standalone time saver (vs. spreadsheet price lists) and a prerequisite for the quote flow.

## Acceptance Criteria

- Catalog page at /catalog route
- Lists all materials for tenant: name, category, unit, price per unit
- Add material: form with name, category, unit, price, depth, supplier SKU
- Edit material: click to edit in-place or modal
- Delete material: confirmation, then remove
- Calls existing material CRUD API routes (from T-004-02)
- Empty state: prompt to add first material
- S.2.2 scenario registered (catalog management)
