---
id: T-009-01
story: S-009
title: material-assignment-ui
type: task
status: open
priority: high
phase: done
depends_on: [T-007-02, T-008-01]
---

## Context

The interface where a landscaper assigns materials to zones per tier. Select a zone, pick a material from the tenant catalog, assign it to good/better/best. This is the core design interaction — it connects the zone editor to the quote engine.

## Acceptance Criteria

- Zone list panel showing all zones with type and label
- Click a zone to select it; highlight on canvas
- Material picker: list from tenant catalog, grouped by category
- Assign material to selected zone for a specific tier (good/better/best)
- Tier tab navigation to see/edit assignments per tier
- Assignments persist via PUT /projects/:id/tiers/:tier
- Quote total updates when assignments change (calls GET /quote/:tier)
