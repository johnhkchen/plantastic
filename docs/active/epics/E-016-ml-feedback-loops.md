---
id: E-016
title: ML Feedback Loops & Data Capture
status: open
priority: medium
sprint: 3
---

## Context

The segmentation pipeline (E-014) uses BAML for feature classification. The trait abstractions (T-033-06) make it swappable. But the missing piece is **data capture at the UX moments where users make decisions.** Every user interaction is a potential training example — corrections, zone placements, material selections, preview behaviors.

This epic doesn't add ML models. It adds the instrumentation that makes ML possible: structured event logging at the 5 key UX moments, correction capture in the viewer, and the data pipeline that turns user decisions into training datasets.

## The 5 UX Moments

### 1. Scan Review — "What am I looking at?"
User sees classified features, corrects mistakes (merge, split, relabel).
**Data captured:** (feature_candidate, model_prediction, user_correction, geometric_context)

### 2. Zone Placement — "Where should I put things?"
System suggests zones, user adjusts or draws their own.
**Data captured:** (site_conditions, suggested_zones, final_zones, adjustments)

### 3. Material Selection — "What should go here?"
System suggests plants/materials, user accepts or swaps.
**Data captured:** (zone_conditions, suggested_materials, final_materials)

### 4. Design Preview — "Does this look right?"
User reviews 3D preview and quote, makes revisions.
**Data captured:** (interaction_traces, dwell_times, post_preview_edits, tier_selected)

### 5. Post-Install — "Did it work?"
Outcome feedback months later.
**Data captured:** (design_decisions, outcome_photos, user_rating, notes)

## What researchers get

Each moment produces a specific dataset type:

| Moment | Dataset | ML Application |
|--------|---------|----------------|
| Scan Review | Hard negative/positive classification examples | Feature classifier distillation |
| Zone Placement | (site → zone) mapping examples | Automated zone suggestion |
| Material Selection | (conditions → materials) preference data | Recommendation system |
| Design Preview | Interaction traces with implicit quality signal | Design quality scoring (RLHF) |
| Post-Install | Full trajectory with ground-truth outcome | End-to-end prediction validation |

## Principles

1. **Capture decisions, not just results.** The delta between suggestion and final choice is more valuable than the final choice alone.
2. **Log the context.** A correction without the geometric/environmental context is useless for training.
3. **Never block the user for data capture.** Logging is async, fire-and-forget. The UI never waits on a write.
4. **Privacy by design.** Logs are per-tenant, exportable, deletable. No cross-tenant data leakage.
5. **Start with the correction loop.** Scan Review corrections (moment 1) have the highest signal-to-noise ratio and feed directly into feature classifier improvement.

## Stories

- S-039: Correction Capture UI (merge/split/relabel in viewer)
- S-040: Classification Event Logging (JSONL pipeline)
- S-041: Zone Placement Telemetry (suggested vs final)
- S-042: Design Session Traces (preview interactions)

## Success Criteria

- Every BAML classification call logged with input + output + context
- User corrections in the viewer captured as (before, after, context) tuples
- Zone placement suggestions tracked against final user decisions
- All logging async, opt-in, per-tenant, GDPR-deletable
- A researcher can `cat data/events/*.jsonl | python analyze.py` and get useful features
- No ML crates added — this is instrumentation only
