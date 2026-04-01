---
id: E-018
title: Quote Comparison in 3D Viewer
status: open
priority: high
sprint: 3
---

## Context

The quote comparison page shows a 2D table of Good/Better/Best with dollar amounts. But the 3D viewer already supports tier switching. This epic adds quote data directly into the viewer experience: see the zone, see the materials, see the price — all in one view. The client-facing comparison (S.3.4) becomes a 3D experience, not just a spreadsheet.

## Architecture

```
Viewer page with tier tabs
  │
  ├─ 3D scene: zones rendered with tier-specific materials
  │    setTier("good") → green zones, basic materials
  │    setTier("better") → richer materials, edging visible
  │    setTier("best") → premium materials, lighting fixtures
  │
  ├─ Quote overlay panel (HTML over iframe, not in Bevy)
  │    Per-zone: material name, quantity, line total
  │    Grand total for current tier
  │    "Compare" button toggles side-by-side summary
  │
  └─ Zone tap → callout popup
       Material name, supplier SKU, unit price, depth
       Photo thumbnail if available
```

## Stories

- S-046: Quote Data in Viewer Page (fetch quotes, display alongside 3D)
- S-047: Zone Tap → Material Callout (tap zone in viewer → see details)
- S-048: Client Share View (S.3.4 — `/c/[token]` route with viewer + quote)

## Success Criteria

- Viewer page shows quote totals per tier alongside 3D scene
- Tapping a zone shows material callout with price
- Tier switching updates both 3D materials and quote numbers
- S.3.4 (Client quote comparison view) passes
- S.4.3 advances integration (callouts visible in viewer, not just API)
