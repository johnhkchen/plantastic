---
id: T-034-01
story: S-034
title: baml-classify-features
type: task
status: open
priority: high
phase: ready
depends_on: [T-033-02]
---

## Context

The BAML ClassifyFeatures function takes geometric FeatureCandidate summaries and returns domain-expert classifications: species identification for trees, type for structures, confidence scores, and landscape design notes.

## Acceptance Criteria

- Add to `baml_src/classify.baml`:
  - ClassifyFeatures function: FeatureCandidate[] + address + climate_zone → ClassifiedFeature[]
  - ClassifiedFeature: cluster_id, label, category (tree/structure/hardscape/planting/utility), species?, confidence, reasoning, landscape_notes
- Regenerate baml_client (`baml-cli generate`)
- Integration in pt-scan or pt-features: `classify_features(candidates, context) -> Vec<ClassifiedFeature>`
- Three generator implementations (same pattern as proposal):
  - BamlFeatureClassifier: real API (production)
  - ClaudeCliClassifier: claude CLI (subscription dev)
  - MockFeatureClassifier: fixture-based (CI)
- Run on Powell & Market candidates, capture output as fixture
- Prompt tuned for SF Bay Area urban landscape:
  - Market Street tree species: London Plane (Platanus × acerifolia), Brisbane Box (Lophostemon confertus)
  - Transit infrastructure: Muni overhead wires, shelters, signal poles
  - Hardscape: concrete sidewalk, granite curb, brick paving
- `just check` passes

## Implementation Notes

- The prompt should include address and climate zone so the LLM can make regional inferences
- Powell & Market: USDA zone 10b, Sunset zone 17, Mediterranean maritime climate
- Confidence should be calibrated: trees with distinctive height/spread profiles get high confidence, ambiguous structures get lower
- The `reasoning` field is the "wow" text: "25-ft height with 20-ft spreading crown matches mature London Plane, the dominant Market Street street tree species"
- landscape_notes is for design implications: "provides afternoon shade for south-facing areas, root zone requires setback for hardscape"
