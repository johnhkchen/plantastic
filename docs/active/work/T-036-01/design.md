# T-036-01 Design: Measure Feature Gaps

## Problem

Given N feature candidates with centroids, bounding boxes, and spreads, compute
pairwise gaps that represent plantable zones between features. The canonical case:
two street tree trunks with a planter zone between them.

## Options Considered

### Option A: New module in pt-scan (`gap.rs`)

Add a `gap.rs` module alongside `feature.rs`. Both are post-clustering geometry
computations. The gap module takes `&[FeatureCandidate]` and `&Plane`, returns
`Vec<Gap>`. Follows the same pattern as `feature.rs`.

**Pros:** Co-located with the data it operates on. Consistent with crate structure.
No cross-crate dependency needed.

**Cons:** pt-scan is growing. But gap measurement is logically part of the scan
analysis pipeline — it belongs here.

### Option B: Separate pt-gap crate

New crate for spatial relationship computations.

**Pros:** Clean boundary.

**Cons:** Overkill for ~100 lines of geometry. Adds a crate just for one function.
The Gap struct needs FeatureCandidate, creating a dependency on pt-scan anyway.

### Option C: Put it in pt-geo

pt-geo is for generic 2D geometry (area, perimeter, boolean ops). Gap measurement
is scan-specific, not generic geometry. Wrong abstraction level.

## Decision: Option A — `gap.rs` in pt-scan

Same pattern as `feature.rs`. The pipeline is: cluster → candidates → gaps.
All three steps live in pt-scan.

## Gap Struct Design

```rust
#[derive(Debug, Clone, Serialize)]
pub struct Gap {
    pub feature_a_id: usize,          // cluster_id of first feature
    pub feature_b_id: usize,          // cluster_id of second feature
    pub centroid_distance_ft: f64,    // center-to-center distance
    pub clear_width_ft: f64,          // plantable width (minus radii)
    pub clear_length_ft: f64,         // perpendicular extent
    pub area_sqft: f64,               // clear_width × clear_length
    pub ground_elevation_ft: f64,     // ground plane height at midpoint
    pub midpoint: [f64; 2],           // XY midpoint of gap (meters)
}
```

### Field Definitions

**centroid_distance_ft:** 2D Euclidean distance between centroids in XY plane,
converted to feet. Z is excluded — vertical separation isn't a gap.

**clear_width_ft:** The actual plantable width between feature envelopes.
`centroid_distance_ft - (spread_a_ft/2 + spread_b_ft/2)`. If negative, features
overlap and no gap exists (filtered out). Uses `spread_ft` from FeatureCandidate
as diameter approximation.

**clear_length_ft:** The extent of the gap perpendicular to the connecting line.
For the planter zone use case, this represents how deep the planter is. Defined as
`min(spread_a_ft, spread_b_ft)` — bounded by the shorter feature's extent. This
is the conservative estimate: the planter can't be deeper than the narrower feature
constrains it.

Rationale: For two tree trunks side by side, the "length" of the planter between
them is roughly the depth of the sidewalk/planter strip, which correlates with the
trees' footprint. Using the min of the two spreads gives a conservative rectangular
approximation.

**area_sqft:** `clear_width_ft × clear_length_ft`. The rectangular plantable area.

**ground_elevation_ft:** Height of the ground plane at the gap midpoint. Computed
by projecting the midpoint onto the ground plane and converting to feet. For
near-horizontal planes (typical LiDAR scans), this is approximately `-d / normal[2]`
evaluated at the midpoint XY.

**midpoint:** XY center of the gap in meters. Useful for downstream visualization
and spatial queries.

## Configuration

```rust
#[derive(Debug, Clone)]
pub struct GapConfig {
    pub max_distance_ft: f64,  // only compute gaps closer than this (default: 30.0)
}
```

Single knob. The ticket specifies 30ft as the example threshold. Gaps beyond this
are not planter zones — they're separate features that happen to be in the same scan.

## Function Signature

```rust
pub fn measure_gaps(
    candidates: &[FeatureCandidate],
    ground_plane: &Plane,
    config: &GapConfig,
) -> Vec<Gap>
```

Returns gaps sorted by centroid_distance_ft (closest first). Filters out:
- Pairs beyond max_distance_ft
- Pairs with negative clear_width (overlapping features)

## Algorithm

1. For each unique pair (i, j) where i < j:
   a. Compute 2D centroid distance in feet
   b. Skip if > max_distance_ft
   c. Compute clear_width = distance - (spread_i/2 + spread_j/2)
   d. Skip if clear_width ≤ 0 (overlapping)
   e. Compute clear_length = min(spread_i, spread_j)
   f. Compute area = clear_width × clear_length
   g. Compute midpoint XY (meters)
   h. Compute ground_elevation at midpoint
2. Sort by centroid_distance_ft ascending
3. Return

Complexity: O(N²) pairwise — N is typically 2-20 features, so this is trivial.

## Ground Elevation Computation

The ground plane is `n·p + d = 0` where `n = [a, b, c]`. Given midpoint (mx, my):

If the plane is near-horizontal (c ≈ 1.0), the ground elevation at (mx, my) is:
`z = -(a*mx + b*my + d) / c`

Edge case: if c ≈ 0, the plane is nearly vertical — degenerate. Return 0.0 and
let downstream handle it. In practice, LiDAR ground planes always have c >> 0.

Convert z from meters to feet for the output.

## Rejected Alternatives

**Using bbox edges instead of spread/2 as radius:** More precise but fragile.
Axis-aligned bounding boxes don't align with the gap axis. A rotated feature would
give misleading edge-to-edge distances. The spread-as-diameter approach is simpler
and robust for roughly circular features (trunks, lamp posts).

**Computing convex hull distances:** Exact edge-to-edge distance using convex hulls
of the point clusters. Much more complex, requires access to raw points. Overkill
for planter estimation where ±6 inches doesn't matter.

**3D distance instead of 2D:** Including Z in the centroid distance would conflate
vertical and horizontal separation. A tall feature next to a short one still has a
horizontal gap — that's what we want to measure.

## Validation

### Unit test (from ticket)
Two cylinders 10ft apart, 2ft diameter each → gap width = 6ft.
- centroid_distance = 10ft
- spread_a = spread_b = 2ft
- clear_width = 10 - (1 + 1) = 8ft... wait.

Re-reading: "Two cylinders 10ft apart, 2ft diameter each → gap width = 6ft"
- 10ft apart means 10ft center-to-center
- 2ft diameter = 1ft radius each
- clear_width = 10 - (1 + 1) = 8ft

But the ticket says 6ft. Let me re-read: "clear_width = centroid distance -
(spread_a/2 + spread_b/2)". With 2ft diameter, spread = 2ft, so spread/2 = 1ft.
clear_width = 10 - (1+1) = 8ft.

Unless "10ft apart" means edge-to-edge, not center-to-center. Then:
- center-to-center = 10 + 1 + 1 = 12ft
- clear_width = 12 - 2 = 10ft. Still not 6ft.

Or: "2ft diameter" means spread_ft = 2ft per the FeatureCandidate definition
(max bbox extent). "10ft apart" = center-to-center = 10ft.
clear_width = 10 - (2/2 + 2/2) = 10 - 2 = 8ft.

The ticket's arithmetic is wrong, or the test intent is different. I'll implement
the formula as documented in acceptance criteria and adjust the test to match:
two features with centroids 10ft apart, spread_ft=2ft each → clear_width = 8ft.

### Powell & Market validation
With 2 trunk clusters, should produce exactly 1 gap with plausible dimensions
(a few feet wide — typical sidewalk planter strip).
