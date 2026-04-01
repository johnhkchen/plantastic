---
id: S-008
epic: E-003
title: Quote API
status: open
priority: high
dependencies:
  - S-004
---

# S-008: Quote API

## Purpose

Wire the pt-quote computation engine to an HTTP route. This takes S.3.1 and S.3.2 from ★☆☆☆☆ (engine works in isolation) to ★★ (reachable via API). A developer can compute quotes with curl; the frontend can fetch them.

## Scope

- GET /projects/:id/quote/:tier endpoint
- Loads project zones from database, tier assignments, material catalog
- Calls pt-quote::compute_quote with real data
- Returns JSON with line items, subtotal, total
- Error handling: missing project, no assignments for tier, invalid tier name
- Upgrade S.3.1/S.3.2 scenario tests to exercise the API route (★★)

## Tickets

- T-008-01: Quote API route
- T-008-02: Scenario upgrade to ★★
