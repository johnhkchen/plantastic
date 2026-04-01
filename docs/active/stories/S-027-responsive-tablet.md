---
id: S-027
epic: E-012
title: Responsive & Tablet Layout
status: open
priority: medium
depends_on: [S-026]
tickets: [T-027-01, T-027-02]
---

## Goal

All frontend pages work on iPad-width viewports (768px–1024px). The quote comparison 3-column grid and zone editor canvas are the biggest challenges. This is required for crew handoff (S.4.1 — tablet in the field).

## Acceptance Criteria

- Quote comparison columns stack or scroll on viewports < 768px
- Zone editor canvas scales to container width
- Catalog grid reflows from 3-col to 2-col to 1-col
- Sidebar collapses to hamburger on mobile
- Touch targets ≥ 44px on interactive elements
- No horizontal overflow on any page at 375px width
