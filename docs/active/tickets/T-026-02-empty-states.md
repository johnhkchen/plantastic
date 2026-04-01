---
id: T-026-02
story: S-026
title: empty-states
type: task
status: open
priority: medium
phase: ready
depends_on: [T-026-01]
---

## Context

When a new tenant starts with zero data, every list page shows nothing — no guidance, no CTA. Empty states are the onboarding experience.

## Acceptance Criteria

- Empty states with CTAs on:
  - Dashboard (no projects): "Create your first project to get started" + create button
  - Catalog (no materials): "Add materials to your catalog before assigning them to zones" + add button
  - Zone editor (no zones): "Click on the plan view to draw your first zone" + visual hint
  - Quote page (no assignments): "Assign materials to zones to generate quotes" + link to assignment page
  - Viewer (no scan): "Upload a LiDAR scan to see a 3D preview" + upload hint
- Empty state components are reusable (`EmptyState.svelte` with icon, message, action slot)
- `just check` passes

## Implementation Notes

- Keep copy concise and action-oriented
- Include a simple illustration or icon (can use emoji or inline SVG for now)
- Empty states should disappear immediately when data exists — no stale empty state after first add
