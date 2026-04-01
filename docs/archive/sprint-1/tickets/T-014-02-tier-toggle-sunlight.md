---
id: T-014-02
story: S-014
title: tier-toggle-sunlight
type: task
status: open
priority: high
phase: done
depends_on: [T-014-01]
---

## Context

Tier toggling and sunlight control — the features that make the viewer a sales tool. The landscaper shows the client: "here's Good, here's Better, here's Best" by switching scenes. The sunlight slider shows "this is what it looks like at 8am vs 4pm."

## Acceptance Criteria

- Host sends setTier(name) → viewer loads the corresponding glTF scene URL
- Scene swap is smooth (fade or quick cut, not a blank frame)
- Camera position preserved across tier switches
- Host sends setLightAngle(degrees) → viewer rotates directional light
- Shadow direction updates in real time as the slider moves
- Viewer sends back current light angle so host can display "2:00 PM" etc.
