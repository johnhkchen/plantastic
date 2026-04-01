---
id: T-034-02
story: S-034
title: annotated-plan-view
type: task
status: open
priority: medium
phase: ready
depends_on: [T-034-01]
---

## Context

The plan-view PNG currently shows raw elevation shading. After feature classification, we can overlay labeled bounding boxes and feature names — turning a grayscale top-down view into an annotated site plan.

## Acceptance Criteria

- Extend pt-scan's `to_plan_view_png()` (or new function) to accept ClassifiedFeature[]
- Overlay per feature:
  - Bounding box outline (color-coded by category: green=tree, gray=structure, tan=hardscape)
  - Label text: feature name + confidence (e.g., "London Plane (0.92)")
  - Optional: category icon (simple shapes — circle for tree, square for structure)
- Output: annotated PNG alongside the plain one
- CLI example extended: produce both plain and annotated plan views
- The annotated PNG is the "wow" screenshot — should look like a professional site survey

## Implementation Notes

- Use the `image` crate's drawing capabilities (already a pt-scan dep)
- imageproc crate may help for text rendering and line drawing
- Font: embed a small TTF (e.g., DejaVu Sans Mono) for label text
- Labels should not overlap — simple collision avoidance (shift down if overlap)
- Color palette: tree=#22c55e, structure=#6b7280, hardscape=#d97706, utility=#ef4444
- This is the image that goes in the sales deck and README
