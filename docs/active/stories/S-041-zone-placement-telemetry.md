---
id: S-041
epic: E-016
title: Zone Placement Telemetry
status: open
priority: medium
depends_on: [S-040]
tickets: [T-041-01]
---

## Goal

When the system suggests zones (from scan gaps or BAML recommendations) and the user adjusts them, capture the delta. This is the dataset for "given these site conditions, where would a landscaper put zones?"

## Acceptance Criteria

- Log event when system suggests a zone (with position, size, type, rationale)
- Log event when user creates/moves/resizes a zone (with final state)
- Compute delta: suggested → final (translation, scale, type change)
- Context includes: scan features, sun grid, climate, existing vegetation
- Researchers can extract: (site_conditions, suggested_zone, final_zone) triples
