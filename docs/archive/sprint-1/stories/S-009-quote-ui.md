---
id: S-009
epic: E-003
title: Quote UI
status: open
priority: high
dependencies:
  - S-007
  - S-008
---

# S-009: Quote UI

## Purpose

The frontend where landscapers assign materials to zones per tier and see the resulting quotes. This is the "time saved" moment — instead of spreadsheets and calculators, they drag a material onto a zone and see the price update.

Takes S.3.1/S.3.2 from ★★ to ★★★.

## Scope

- Material assignment interface: select zone, pick material from catalog, assign per tier
- Three-tier comparison view: Good / Better / Best side by side with line items and totals
- Real-time recalculation when assignments change
- Upgrade S.3.1/S.3.2 scenario tests to ★★★ (tested through frontend flow)

## Tickets

- T-009-01: Material assignment UI per zone per tier
- T-009-02: Three-tier comparison page + scenario ★★★
