---
id: T-027-02
story: S-027
title: responsive-editor-viewer
type: task
status: open
priority: medium
phase: ready
depends_on: [T-027-01]
---

## Context

The zone editor canvas has a fixed aspect ratio that doesn't adapt to container width. The Bevy viewer iframe has similar constraints. Both need to work on tablet for crew handoff (S.4.1).

## Acceptance Criteria

- Zone editor:
  - Canvas scales to fill available width (preserving aspect ratio)
  - Measurements panel moves below canvas on narrow viewports
  - Touch events work for vertex placement and dragging (pointer events, not just mouse)
- Viewer:
  - Iframe fills container responsively
  - Controls (tier toggle, sunlight slider) reflow below viewer on narrow viewports
  - Touch orbit/pan/zoom already supported by bevy_panorbit_camera (T-014-01)
- Sidebar:
  - Collapses to hamburger menu on < 768px
  - Overlay when open, not push-content
- `just check` passes
