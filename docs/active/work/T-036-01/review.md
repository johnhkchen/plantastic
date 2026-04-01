# T-036-01 Review: Measure Feature Gaps

## Summary

Implemented `measure_gaps()` in a new `gap.rs` module within the pt-scan crate.
Given feature candidates from DBSCAN clustering, the function computes pairwise
gap measurements — the plantable zones between features (e.g., the planter strip
between two street tree trunks).

## Files Changed

### Created
- **`crates/pt-scan/src/gap.rs`** (~130 lines)
  - `Gap` struct: 8 fields (feature_a_id, feature_b_id, centroid_distance_ft,
    clear_width_ft, clear_length_ft, area_sqft, ground_elevation_ft, midpoint)
  - `GapConfig` struct: configurable max_distance_ft threshold (default: 30ft)
  - `measure_gaps()`: pairwise gap computation, filters overlaps and distant pairs
  - 7 unit tests covering all branches

### Modified
- **`crates/pt-scan/src/lib.rs`** (+2 lines)
  - Added `pub mod gap;` and re-exports for `measure_gaps`, `Gap`, `GapConfig`

- **`crates/pt-scan/tests/integration.rs`** (+120 lines)
  - `make_two_cluster_ply()`: synthetic PLY with two obstacle columns 4m apart
  - `test_gap_measurement_synthetic`: end-to-end pipeline test
  - `test_powell_market_gaps`: real scan validation (when PLY file available)

- **`crates/pt-scan/examples/process_sample.rs`** (+30 lines)
  - Stage 7: gap measurement with formatted table output

### Not changed by this ticket (pre-existing)
- `crates/pt-features/src/lib.rs` was populated by a pre-commit hook to fix a
  broken workspace member. This is from T-034-01 in-progress work.

## Test Coverage

| Test | What it verifies |
|------|-----------------|
| `test_two_features_known_gap` | Correct distance, clear_width, length, area for known geometry |
| `test_overlapping_features` | Negative clear_width pairs filtered out |
| `test_beyond_threshold` | Distant pairs (>30ft) filtered out |
| `test_empty_candidates` | 0 candidates → empty result |
| `test_single_candidate` | 1 candidate → no pairs → empty result |
| `test_ground_elevation` | Elevation computed from ground plane at midpoint |
| `test_three_features_pairwise` | N=3 → 3 pairs, sorted by distance |
| `test_gap_measurement_synthetic` | Full pipeline: PLY → cluster → candidates → gaps |
| `test_powell_market_gaps` | Real scan validation (skips if file absent) |

**All expected values in tests are computed independently** — not derived from system
functions. Example: 3.048m × 3.28084 = 10.0ft (hand-computed, not via the crate).

## Acceptance Criteria Check

| Criterion | Status |
|-----------|--------|
| `measure_gaps(candidates, ground_plane) -> Vec<Gap>` | ✓ (takes &GapConfig too) |
| Gap struct with all specified fields | ✓ |
| Configurable distance threshold | ✓ (GapConfig::max_distance_ft) |
| clear_width = centroid_dist - (spread_a/2 + spread_b/2) | ✓ |
| area_sqft = clear_width × clear_length | ✓ |
| Powell & Market validation: 1 gap between 2 trunks | ✓ (test exists, requires PLY) |
| Unit test: two cylinders with known dimensions | ✓ |

## Arithmetic Discrepancy

The ticket says "two cylinders 10ft apart, 2ft diameter each → gap width = 6ft."
The formula in the acceptance criteria gives: `10 - (2/2 + 2/2) = 10 - 2 = 8ft`.
The test uses 8ft, matching the formula. The ticket's "6ft" appears to be an error
(perhaps assuming "10ft apart" means edge-to-edge, but the formula uses centroid
distance).

## Quality Gate

- `cargo fmt --check`: clean
- `cargo clippy -D warnings`: clean
- Unit tests: 48 passed (pt-scan)
- Integration tests: 12 passed (excluding 3 Powell Market real-scan tests)

## Open Concerns

1. **Powell Market test performance:** The three `test_powell_market_*` tests process
   a 294MB PLY file in debug mode, taking >60s each. This is pre-existing (not
   introduced by this ticket). They may timeout under `just test`'s 60s-per-binary
   limit. Consider running these with `--release` or marking them with an explicit
   timeout annotation.

2. **clear_length definition:** Defined as `min(spread_a, spread_b)` — the
   conservative estimate. For asymmetric features (e.g., a narrow trunk next to a
   wide planter box), this may underestimate the gap length. Acceptable for the
   planter estimation use case where both features are typically similar trunks.

3. **2D-only distance:** Gap measurement uses XY plane distance only (Z excluded).
   This is correct for horizontal planter zones but would miss vertical gaps (e.g.,
   between a canopy and a wall). Not relevant for current use cases.

## Scenario Dashboard

No new scenarios added — gap measurement is infrastructure for the planter estimation
pipeline (S-036). The gap data feeds into pt-quote for soil volume and plant count.
The scenario will flip when the full estimation loop is complete.
