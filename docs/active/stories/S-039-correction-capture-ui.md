---
id: S-039
epic: E-016
title: Correction Capture UI
status: open
priority: high
tickets: [T-039-01, T-039-02]
---

## Goal

The Bevy viewer (or a plan-view overlay) lets the user correct feature classifications. These corrections are the highest-value training data — every one is a labeled example with full geometric context.

## Interactions

- **Merge:** Select two clusters → "These are the same object" → clusters merge, label updated
- **Split:** Select one cluster → "This is two things" → re-cluster just that region
- **Relabel:** Select a cluster → change its label from dropdown (tree/hardscape/structure/planting)
- **Confirm:** Implicitly — user moves to the next step without editing (positive example)

## Acceptance Criteria

- Correction UI accessible from scan review page
- Each correction logged as a structured event with before/after state
- Corrections persist (saved to project, not ephemeral)
- Re-running classification on the same scan starts from corrections, not from scratch
- Works on plan-view overlay (simpler, immediate) — 3D viewer corrections are a stretch goal
