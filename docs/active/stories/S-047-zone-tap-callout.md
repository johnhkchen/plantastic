---
id: S-047
epic: E-018
title: Zone Tap → Material Callout
status: open
priority: high
depends_on: [S-046]
tickets: [T-047-01]
---

## Goal

Tapping a zone in the 3D viewer shows a callout popup with material details: name, supplier SKU, unit price, install depth, quantity, line total. This is S.4.3 at full integration — the crew foreman taps a zone and sees everything they need.

## Acceptance Criteria

- zoneTapped postMessage triggers callout popup in SvelteKit (HTML overlay)
- Callout shows: material name, category, SKU, price/unit, quantity, line total, depth
- Photo thumbnail if material has photo_ref
- Callout dismisses on tap elsewhere or Escape
- S.4.3 advances to ★★★☆☆+ (viewer + callout UI)
