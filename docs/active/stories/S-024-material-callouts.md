---
id: S-024
epic: E-011
title: Crew Handoff — Material Callouts
status: open
priority: high
tickets: [T-024-01]
---

## Goal

Flip S.4.3 (Material callouts with supplier info) from NotImplemented to Pass. Both named prereqs (pt-materials, Bevy viewer) are already delivered.

## What S.4.3 Proves

A landscaper assigns materials to zones → the system exposes per-zone callout data including product name, supplier SKU, install depth, and photo ref. This is the data a crew foreman needs when standing in the yard.

## Acceptance Criteria

- S.4.3 passes at ★☆☆☆☆ (computation: material callout fields verified) or ★★☆☆☆ (API: callout data fetched via materials API)
- Test constructs zones + materials + assignments, verifies callout fields are present and correct
- No new crates — uses existing pt-materials and plantastic-api
- `just check` passes
