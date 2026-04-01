---
id: S-045
epic: E-017
title: Tier Switching on Scan Scene
status: open
priority: high
depends_on: [S-044]
tickets: [T-045-01]
---

## Goal

Good/Better/Best tier toggle changes the materials rendered in zone overlays. The terrain stays the same; the planter zone between the trunks switches from concrete pavers (Good) to travertine (Better) to flagstone (Best) — different colors/textures per tier.

## Acceptance Criteria

- Tier toggle (existing setTier postMessage) swaps zone material appearance
- Material color derived from category (or material-specific PBR when available)
- Transition is instant (keep-until-ready pattern from T-014-02)
- Quote totals update alongside the 3D view
