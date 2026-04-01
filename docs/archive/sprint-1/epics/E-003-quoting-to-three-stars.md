---
id: E-003
title: Quoting Pipeline to ★★★
status: open
sprint: 2
---

# E-003: Quoting Pipeline to ★★★

## Goal

A landscaper can draw zones on a project, assign materials per zone per tier, and view a three-tier quote comparison in the UI. The quote engine (already passing at ★☆☆☆☆) gets wired through the API and into a usable frontend.

This is the fastest path to meaningful effective savings — 40 raw minutes are already proven, they just need integration.

## Target

- S.3.1 Quantity computation: ★☆☆☆☆ → ★★★ (5.0 → 15.0 effective min)
- S.3.2 Three-tier quotes: ★☆☆☆☆ → ★★★ (3.0 → 9.0 effective min)
- S.2.1 Zone drawing: — → ★★ (0.0 → 8.0 effective min)

## Stories

- **S-007**: Zone drawing — canvas polygon editor + API persistence
- **S-008**: Quote API — wire pt-quote to HTTP routes
- **S-009**: Quote UI — material assignment + three-tier comparison page

## Success Criteria

- Landscaper draws zones on a plan view in the browser
- Zones persist via API and display computed area/perimeter
- Landscaper assigns materials to zones per tier from their catalog
- Three-tier quote comparison renders with correct line items and totals
- S.3.1, S.3.2 scenarios pass at ★★★ (tested through UI flow)
- S.2.1 scenario passes at ★★ (zone drawing + measurements via API)
