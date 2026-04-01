---
id: S-034
epic: E-014
title: BAML Feature Classification
status: open
priority: high
depends_on: [S-033]
tickets: [T-034-01, T-034-02]
---

## Goal

Use BAML to classify geometric feature candidates with landscaping domain knowledge. The LLM sees structured summaries (height, spread, color, shape) and returns species/type identification, confidence, and landscape design notes.

## Acceptance Criteria

- BAML schema: ClassifyFeatures function with FeatureCandidate[] input, ClassifiedFeature[] output
- ClaudeCliGenerator integration (subscription, zero API cost for dev)
- Mock fixture from real classification of Powell & Market features
- Address + climate zone context improves classification accuracy
- Scenario test: synthetic features → classification → verify structure
