# T-033-02 Design: Feature Candidates

## Decision: New `feature.rs` Module

### Options Considered

**A. Extend cluster.rs with extraction logic**
- Pro: keeps cluster-related code together
- Con: cluster.rs is already 360 lines; mixing clustering algorithm with feature
  extraction conflates two distinct responsibilities

**B. New `feature.rs` module** (chosen)
- Pro: clear separation — clustering is spatial grouping, feature extraction is
  geometric summarization. Each module has one job.
- Pro: `FeatureCandidate` is a BAML-facing contract; isolating it makes the
  schema boundary visible
- Pro: easier to test independently with synthetic clusters
- Con: one more file — acceptable for a distinct responsibility

**C. Separate crate (pt-feature)**
- Pro: strong boundary
- Con: overkill for ~200 lines; FeatureCandidate needs direct access to Point,
  Cluster, Plane types — cross-crate dependency adds friction for no benefit

### Chosen: Option B

New module `crates/pt-scan/src/feature.rs` containing `FeatureCandidate` struct
and `extract_candidates()` function.

## FeatureCandidate Struct Design

```rust
#[derive(Debug, Clone, Serialize)]
pub struct FeatureCandidate {
    pub cluster_id: usize,
    pub centroid: [f64; 3],
    pub bbox_min: [f64; 3],
    pub bbox_max: [f64; 3],
    pub height_ft: f64,
    pub spread_ft: f64,
    pub point_count: usize,
    pub dominant_color: String,
    pub vertical_profile: String,
    pub density: f64,
}
```

Matches ticket acceptance criteria exactly. Uses `f64` for BAML compatibility
(JSON numbers). `Serialize` only — BAML sends this to the LLM, never deserializes
back into this struct.

## Function Signature

```rust
pub fn extract_candidates(
    clusters: &[Cluster],
    points: &[Point],
    ground_plane: &Plane,
) -> Vec<FeatureCandidate>
```

Why `points` is a separate parameter: `Cluster.point_indices` indexes into the
obstacle point slice. The caller passes `&cloud.obstacles`. This avoids storing
points inside Cluster (which would double memory for large scans).

## Height Calculation

```
height = max over cluster points of |normal · position + d|
```

Uses the full plane equation, not raw Z. Converted to feet: `* 3.28084`.

**Why max, not centroid height:** The ticket says "max_z - ground_plane_z at centroid"
but for a tilted ground plane, per-point signed distance is more correct. We take
the max distance among all cluster points as the feature height.

## Spread Calculation

```
spread = max(bbox_width_x, bbox_width_y)
```

Horizontal extent only (XY plane). Z extent is captured by height.
Converted to feet.

## Density

```
volume = dx * dy * dz  (clamped to min 0.001 per axis)
density = point_count / volume
```

Clamping prevents division by zero for flat/linear clusters.

## Color Classification

Strategy: count RGB values per cluster, classify the mean color.

Buckets (by hue/saturation/value analysis of mean RGB):
- **green**: G channel dominant, G > 80, G > R*1.2, G > B*1.2
- **brown**: R > G, R > 80, G > 40, B < G (earth tones)
- **gray**: all channels within 30 of each other, value > 50
- **white**: all channels > 200
- **mixed**: none of the above

When no points have color data → "unknown".

This is deliberately coarse per the ticket: "Color names are intentionally coarse —
the LLM interprets fine distinctions."

## Vertical Profile Classification

Per ticket spec:
- `height/spread > 3` → **columnar** (tall and narrow, like poles)
- `height/spread < 0.5` → **flat** (low and wide, like planters)
- Otherwise → **spreading** (moderate aspect ratio, like trees)

The ticket also mentions "conical" and "irregular" in the acceptance criteria enum.
Decision: implement the three primary categories from the implementation notes.
Add conical detection as a refinement of columnar/spreading:

- **conical**: height/spread between 0.5 and 3, AND upper half of cluster is
  narrower than lower half (taper ratio < 0.7). Characteristic of evergreen trees.
- **irregular**: catch-all when height or spread is near zero (degenerate geometry)

Priority order: irregular → columnar → flat → conical → spreading.

## CLI Extension

Add clustering + feature extraction as stage 6 in `process_sample.rs`:
1. Run `cluster_obstacles` on `cloud.obstacles`
2. Run `extract_candidates` on clusters
3. Print a formatted table of candidates

## Testing Strategy

### Unit tests (in feature.rs)
- Synthetic cluster with known geometry → verify all fields
- Color classification: pure green, brown, gray, white, mixed inputs
- Vertical profile: columnar (tall/narrow), flat (wide/low), spreading, conical
- Edge case: single-point cluster (degenerate bbox)
- Edge case: no color data → "unknown"

### Integration tests (tests/integration.rs)
- End-to-end: synthetic PLY → process → cluster → extract → verify candidate count
- Powell & Market scan → extract → expect 2+ candidates with reasonable values

## Rejected Alternatives

- **PCA for vertical profile**: Principal component analysis would give more precise
  shape classification but adds complexity (nalgebra eigendecomposition). The simple
  ratio heuristic is sufficient for coarse LLM input. Can upgrade later if needed.

- **Per-point ground distance for height**: Computing distance for every point and
  taking max is O(n). Could approximate with bbox corners, but real scans have
  irregular shapes — max over all points is more accurate and n is small per cluster.

- **HSV color space**: Converting RGB→HSV for classification would be more robust
  but adds complexity. Mean RGB with threshold-based classification is adequate for
  the five coarse categories.
