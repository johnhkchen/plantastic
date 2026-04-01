---
id: S-048
epic: E-018
title: Client Share View
status: open
priority: medium
depends_on: [S-046, S-047]
tickets: [T-048-01, T-048-02]
---

## Goal

S.3.4 — a `/c/[token]` route where the homeowner sees Good/Better/Best with the 3D viewer and quote numbers. No login required. The landscaper sends a link; the client compares tiers and picks one.

## Acceptance Criteria

- Share token generated per project (random, unguessable)
- `/c/[token]` loads: 3D viewer + tier tabs + quote summary
- Read-only — no editing, no zone drawing
- Responsive for mobile (homeowner opens on phone)
- Optional: client can mark "I'm interested in [tier]" → notification to landscaper
- S.3.4 passes at ★★★☆☆
