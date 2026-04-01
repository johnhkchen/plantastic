# T-035-02 Research: Reconcile Site Data

## Objective

Map codebase state relevant to building `ReconcileSiteData` — the capstone BAML
function that merges scan features, satellite baseline, and plan-view analysis
into a single reconciled site model.

---

## Data Sources Feeding Reconciliation

### 1. Scan Features (pt-scan → pt-features)

**Origin:** iPhone LiDAR point cloud → DBSCAN clustering → feature extraction → BAML classification.

**Type:** `ClassifiedFeature` (classify.baml:17–25)
- cluster_id, label, category, species?, confidence, reasoning, landscape_notes
- Categories: tree, structure, hardscape, planting, utility
- Confidence 0.0–1.0 with calibration guidance in prompt

**Pipeline:** `PointCloud → cluster() → extract_candidates() → ClassifyFeatures() → ClassifiedFeature[]`

Scan features represent **ground truth at time of scan** — what the LiDAR physically detected.
They include geometry (height, spread, profile) but the ClassifiedFeature struct strips
raw geometry in favor of semantic labels.

### 2. Satellite Baseline (pt-satellite)

**Origin:** Address → geocode → parcel boundary → canopy detection → sun exposure grid.

**Type:** `ProjectBaseline` (crates/pt-satellite/src/types.rs)
- coordinates: WGS84 lat/lng
- lot_boundary: Polygon<f64> + area_sqft + DataSourceLabel
- trees: Vec<DetectedTree> (location, height_ft, spread_ft, confidence)
- sun_grid: ExposureGrid (pt-solar)

**Key difference from scan:** Satellite data is **pre-visit** — it represents what remote
sensing knew before anyone walked the site. Trees are detected from canopy height models,
not LiDAR. No hardscape/utility detection. Confidence is lower and species ID is absent.

Currently only `EmbeddedSource` exists (hardcoded SF addresses: 1234 Noriega St, etc.).
Powell & Market is the standard test scenario.

### 3. Plan-View Analysis (pt-analyzer, T-035-01)

**Origin:** Annotated plan-view PNG + classified features → BAML `AnalyzePlanView`.

**Type:** `SiteAnalysis` (analyze.baml:16–20)
- features: ClassifiedFeature[] (echo of input)
- suggested_zones: Vec<SuggestedZone> (label, zone_type, rationale, area_sqft)
- site_observations: Vec<SiteObservation> (free-form insights)

This is the **spatial reasoning layer** — it looks at the plan view image and classified
features together to suggest zone placements and design observations. It does NOT
cross-reference satellite data.

---

## Existing Crate Pattern (Three-Tier)

All BAML-backed crates follow identical structure:

```
crates/pt-{name}/
├── Cargo.toml
├── src/
│   ├── lib.rs          # baml_client include, module declarations, re-exports
│   ├── {trait}.rs      # Trait + BamlXxx impl
│   ├── claude_cli.rs   # ClaudeCliXxx impl (text-only fallback)
│   ├── mock.rs         # MockXxx + MockFailingXxx + fixture function
│   └── error.rs        # Error enum via thiserror
```

Examples: pt-features, pt-planter, pt-analyzer.

Key conventions:
- `#[path = "../../../baml_client/mod.rs"] mod baml_client;` for generated code
- Trait is `async_trait`, method takes `&self` + input struct → `Result<Output, Error>`
- BamlXxx calls `crate::B.FunctionName.call(...)` 
- ClaudeCliXxx builds text prompt, calls `claude` CLI, parses with `crate::B.FunctionName.parse()`
- MockXxx returns deterministic `powell_market_fixture()`
- MockFailingXxx returns `Error::Operation("mock LLM failure: rate limit exceeded")`

---

## BAML Conventions

- Client: `ProposalFallback` (Haiku → Sonnet with exponential retry)
- Prompt: `#"..."#` multi-line, uses `{{ variable }}` and `{{ ctx.output_format }}`
- Types can cross-reference across .baml files (e.g., analyze.baml uses ClassifiedFeature from classify.baml)
- Test blocks embedded in .baml file with `test name { functions [...] args {...} }`
- Image type: `image` in BAML, maps to `baml::Image` in Rust

---

## What Reconciliation Must Produce (from Acceptance Criteria)

**ReconciledSite:**
- confirmed_features[] — seen in both scan and satellite
- scan_only_features[] — scan detected but satellite didn't
- satellite_only_features[] — satellite detected but scan didn't (possible removal)
- discrepancies[] — conflicts between sources, with explanation and design implication
- recommended_zones[] — informed by ALL data sources (sun, features, grade, access)

**Discrepancy type:** description, possible_explanation, design_implication

**Key intelligence:** Temporal reasoning — "satellite shows a tree here but scan doesn't — was it removed recently?"

---

## Input Contract for Reconciliation

The function needs:
1. `scan_features: ClassifiedFeature[]` — from pt-features pipeline
2. `satellite_baseline: ProjectBaseline` — from pt-satellite builder
3. `plan_view_analysis: SiteAnalysis` — from pt-analyzer (T-035-01)

Challenge: `ProjectBaseline` contains complex types (Polygon<f64>, ExposureGrid) that
don't map cleanly to BAML. BAML needs string/number/bool/array primitives.
The satellite data must be **serialized to text** before entering the prompt.

---

## Constraints and Risks

1. **Prompt size:** This function sees the most context of any BAML call. Three full
   data sources plus instructions. Must keep focused to avoid degraded output quality.

2. **Type bridging:** Satellite types (geo::Polygon, ExposureGrid) have no BAML
   representation. Need a summary/serialization layer.

3. **No real satellite data for Powell & Market:** EmbeddedSource has 1234 Noriega St
   but the standard test scenario is Powell & Market. Mock fixture must be self-consistent.

4. **Cross-file type references:** reconcile.baml can reference ClassifiedFeature,
   SuggestedZone, SiteObservation from existing .baml files. New types (ReconciledSite,
   Discrepancy) go in reconcile.baml.

5. **Crate naming:** Following convention → `pt-reconciler` (verb-er pattern like
   pt-analyzer, pt-planter).

---

## Adjacent Work

- T-035-01 (dependency, delivered): pt-analyzer crate + AnalyzePlanView BAML function
- No other open tickets modify baml_src/ or create new LLM-backed crates
- Workspace Cargo.toml will need pt-reconciler added to members
