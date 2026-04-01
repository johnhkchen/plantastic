# T-033-02 Plan: Feature Candidates

## Step 1: Create `feature.rs` with FeatureCandidate struct

- Define `FeatureCandidate` with all 10 fields per ticket AC
- Derive `Debug, Clone, Serialize`
- Implement `extract_candidates()` shell that iterates clusters

Verify: compiles with `cargo check -p pt-scan`

## Step 2: Implement height and spread computation

- `compute_height_ft`: iterate `point_indices`, compute `|n·p + d|` for each,
  take max, multiply by 3.28084
- `compute_spread_ft`: `max(bbox_dx, bbox_dy) * 3.28084`
- Wire into `extract_candidates`

Verify: unit test with known geometry — 8-point cube at known height above z=0 plane

## Step 3: Implement density computation

- `compute_density`: `point_count / (dx * dy * dz)` with 0.01m minimum per axis
- Wire into `extract_candidates`

Verify: unit test — 100 points in 1m cube → density = 100.0

## Step 4: Implement color classification

- `classify_color`: compute mean R, G, B across points with `color: Some(_)`
- Apply threshold rules: green/brown/gray/white/mixed/unknown
- Wire into `extract_candidates`

Verify: unit tests for each color category + no-color-data case

## Step 5: Implement vertical profile classification

- `classify_vertical_profile`: compute height/spread ratio
- Ratio > 3 → "columnar", < 0.5 → "flat"
- For mid-range: check taper (upper-half XY spread vs lower-half) → "conical" if
  tapered, else "spreading"
- Degenerate (spread ≈ 0) → "irregular"

Verify: unit tests for each profile type

## Step 6: Wire module into lib.rs

- Add `pub mod feature;`
- Add `pub use feature::{FeatureCandidate, extract_candidates};`

Verify: `cargo check -p pt-scan`

## Step 7: Add unit tests in feature.rs

- `test_known_cube_candidate`: 8-point cube, known height/spread/density
- `test_color_classification_green`: all-green points → "green"
- `test_color_classification_brown`: earthy tones → "brown"
- `test_color_classification_no_color`: None colors → "unknown"
- `test_vertical_profile_columnar`: tall narrow cluster
- `test_vertical_profile_flat`: wide low cluster
- `test_vertical_profile_spreading`: moderate aspect ratio
- `test_empty_clusters`: empty slice → empty vec

Verify: `cargo test -p pt-scan`

## Step 8: Extend CLI example

- Import clustering and feature types
- After terrain export, run `cluster_obstacles` + `extract_candidates`
- Print table with headers: ID, Height(ft), Spread(ft), Points, Color, Profile, Density

Verify: `cargo build -p pt-scan --example process_sample`

## Step 9: Add integration tests

- `test_feature_candidates_synthetic`: full pipeline with synthetic PLY →
  verify candidate count matches cluster count, fields are reasonable
- `test_powell_market_candidates`: real scan → 2+ candidates with height > 0,
  valid color strings, valid profiles

Verify: `cargo test -p pt-scan --test integration`

## Step 10: Quality gate

- `just check` — fmt, lint, test, scenarios
- Fix any clippy warnings or test failures

## Testing Strategy Summary

| Test | Type | What it verifies |
|------|------|-----------------|
| test_known_cube_candidate | Unit | All fields computed correctly from known geometry |
| test_color_* | Unit | Color classification thresholds |
| test_vertical_profile_* | Unit | Profile ratio heuristics |
| test_empty_clusters | Unit | Edge case handling |
| test_feature_candidates_synthetic | Integration | End-to-end pipeline produces valid candidates |
| test_powell_market_candidates | Integration | Real scan data produces reasonable results |
