---
id: S-042
epic: E-016
title: Design Session Traces
status: open
priority: low
depends_on: [S-041]
tickets: [T-042-01]
---

## Goal

Capture interaction traces during design preview: which tiers the user reviews, how long they spend, what they change after seeing the 3D preview. This is the implicit quality signal — RLHF for landscape design.

## Acceptance Criteria

- Log tier-switch events with timestamps (dwell time per tier)
- Log camera interactions (orbit angle changes, zoom level) — summarized, not raw
- Log post-preview edits (user saw the 3D view, went back and changed materials)
- Log which tier the client ultimately accepts (if tracked)
- All traces anonymized and aggregatable across projects
