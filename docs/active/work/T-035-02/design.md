# T-035-02 Design: Reconcile Site Data

## Decision 1: How to represent satellite data in BAML

### Options

**A. Pass ProjectBaseline as serialized JSON string.**
The Rust trait serializes the full struct to JSON, BAML function takes a `string` param.
Pro: zero information loss. Con: massive prompt token usage (polygon coordinates alone
can be hundreds of tokens), LLM must parse JSON within JSON.

**B. Create a BAML summary type that flattens satellite data to LLM-relevant fields.**
Define `SatelliteBaseline` class in BAML with: lot_area_sqft, tree_count,
detected_trees[] (height, spread, confidence), avg_sun_hours, address/coordinates.
The Rust layer converts ProjectBaseline → SatelliteBaseline before calling BAML.

Pro: focused prompt, no wasted tokens on polygon vertices or grid cells.
Con: lossy — drops raw polygon and per-cell sun data.

**C. Pass satellite data as structured text in the prompt itself.**
No BAML type — just format a text block in the prompt template.

Pro: simplest BAML. Con: no type safety, harder to test.

**Decision: B.** The LLM doesn't need polygon vertices or per-cell exposure values.
It needs lot area, tree inventory, and average sun exposure — the summary is what
drives reasoning. The Rust conversion layer is trivial and keeps the prompt focused
(ticket notes: "this function sees the most context — keep the prompt focused").

---

## Decision 2: BAML output types

### ReconciledSite structure

Per acceptance criteria, the output is:
```
ReconciledSite {
  confirmed_features: ReconciledFeature[]
  scan_only_features: ReconciledFeature[]
  satellite_only_features: ReconciledFeature[]
  discrepancies: Discrepancy[]
  recommended_zones: RecommendedZone[]
}
```

**Sub-decision: ReconciledFeature vs reusing ClassifiedFeature.**

ClassifiedFeature has cluster_id, label, category, species?, confidence, reasoning,
landscape_notes. For satellite-only features, there's no cluster_id or scan reasoning.

**Decision:** Define `ReconciledFeature` with: label, category, source (scan/satellite/both),
confidence, reasoning. This is cleaner than overloading ClassifiedFeature with nullable
scan-specific fields. The LLM produces the reasoning about why this feature is confirmed,
scan-only, or satellite-only.

**Sub-decision: RecommendedZone vs reusing SuggestedZone.**

SuggestedZone from analyze.baml has: label, zone_type, rationale, approximate_area_sqft.
RecommendedZone needs the same fields PLUS sun_exposure_hours and data_sources_used
(which data informed this recommendation). Better to define a new type that extends
the concept rather than reuse and overload.

**Decision:** New `RecommendedZone` type with: label, zone_type, rationale,
approximate_area_sqft, sun_exposure_hours (int?), data_sources (string — e.g.,
"scan, satellite, plan_view").

---

## Decision 3: Crate name and location

Following the established pattern: `pt-reconciler` in `crates/pt-reconciler/`.
Verb-er suffix matches pt-analyzer, pt-planter. Trait name: `SiteReconciler`.

---

## Decision 4: Trait input struct

```rust
pub struct ReconcilerInput {
    pub scan_features: Vec<ClassifiedFeature>,
    pub satellite_baseline: SatelliteBaseline,  // BAML type, not ProjectBaseline
    pub plan_view_analysis: SiteAnalysis,
    pub address: String,
}
```

The Rust layer includes a conversion function:
`pub fn summarize_baseline(baseline: &ProjectBaseline) -> SatelliteBaseline`

This lives in pt-reconciler (not pt-satellite) because it's a presentation concern —
how to summarize satellite data for LLM consumption.

**Why address is separate:** It's needed for the prompt's regional context but isn't
structurally part of any single data source.

---

## Decision 5: Prompt strategy

The prompt must reason about temporal changes (satellite = older, scan = current).
Structure:

1. **Role:** Senior landscape architect with GIS expertise
2. **Context:** Address, satellite baseline summary, scan features, plan-view analysis
3. **Task:** Reconcile sources, detect discrepancies, recommend zones
4. **Feature matching guidance:** Match by category + approximate location/size, not exact coords
5. **Temporal reasoning:** Satellite data is historical; scan is current ground truth
6. **Zone recommendations:** Must reference sun exposure, existing features, grade, access
7. **Output format:** `{{ ctx.output_format }}`

Key: Keep each data source block concise. The plan_view_analysis contains suggested_zones
and observations — the LLM should build on these, not duplicate them.

---

## Decision 6: Satellite summary for BAML

BAML `SatelliteBaseline` class:
```
class SatelliteTree {
  height_ft float
  spread_ft float
  confidence float
}

class SatelliteBaseline {
  lot_area_sqft float
  trees SatelliteTree[]
  avg_sun_hours float
}
```

Drop: polygon geometry, raw ExposureGrid cells, Coordinates (address covers location).
Keep: lot area (design constraint), trees (for matching), average sun hours (zone recs).

The conversion function computes avg_sun_hours from ExposureGrid.values mean.

---

## Decision 7: Mock fixture

Powell & Market scenario, consistent with existing mocks:
- 2 London Planes in scan → both confirmed (satellite also shows 2 trees at similar height)
- 1 satellite tree not found in scan → satellite_only (possible recent removal)
- Fire hydrant scan-only (satellite can't detect small hardscape)
- 1 discrepancy: tree height difference (25ft scan vs 30ft satellite — growth or measurement error)
- 4 recommended zones building on the plan-view suggestions but now informed by sun data

---

## Rejected Alternatives

1. **Putting reconciliation logic in pt-analyzer:** Would overload a single crate.
   Reconciliation is a distinct capability with different inputs.

2. **Having BAML directly reference ProjectBaseline fields:** BAML can't handle
   geo::Polygon or ExposureGrid. The summary layer is necessary.

3. **Reusing SuggestedZone for recommended_zones:** Loses the data-source attribution
   that makes reconciled zones more valuable than single-source suggestions.
