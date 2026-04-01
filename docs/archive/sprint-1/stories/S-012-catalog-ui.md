---
id: S-012
epic: E-005
title: Material Catalog UI
status: open
priority: high
dependencies:
  - S-006
---

# S-012: Material Catalog UI

## Purpose

Landscapers need to manage their material catalog — the products they sell, at their prices, from their suppliers. This is both a standalone capability (saves time vs. spreadsheet price lists) and a prerequisite for the quote flow (material assignment needs a catalog to pick from).

## Scope

- Catalog page: list all materials for the tenant with name, category, unit, price
- Add material form: all fields from pt-materials (name, category, unit, price, depth, supplier SKU, extrusion behavior)
- Edit material: inline or modal editing
- Delete material: confirmation dialog
- Search by name
- Filter by category (hardscape, softscape, edging, fill)
- Targets S.2.2 scenario at ★★

## Tickets

- T-012-01: Catalog management CRUD page
- T-012-02: Search, filter, and category organization
