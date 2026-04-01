# T-036-01 Progress: Measure Feature Gaps

## Completed

### Step 1: Create gap.rs ✓
- Created `crates/pt-scan/src/gap.rs` with `Gap`, `GapConfig`, `measure_gaps()`
- Pure geometry: 2D centroid distance, clear width/length, area, ground elevation
- 7 unit tests all passing

### Step 2: Register in lib.rs ✓
- Added `pub mod gap;` and re-exports: `measure_gaps`, `Gap`, `GapConfig`

### Step 3: Unit tests ✓
- test_two_features_known_gap: 10ft apart, 2ft spread → 8ft clear width
- test_overlapping_features: filtered out (negative clear_width)
- test_beyond_threshold: filtered out (>30ft)
- test_empty_candidates / test_single_candidate: empty results
- test_ground_elevation: plane at z=1m → 3.28ft elevation
- test_three_features_pairwise: 3 features → 3 gaps, sorted by distance

### Step 4: Integration test ✓
- Added `make_two_cluster_ply()` — synthetic PLY with two obstacle columns 4m apart
- `test_gap_measurement_synthetic`: full pipeline → verifies gap distance/width/area
- `test_powell_market_gaps`: validates gaps on real scan file (when available)

### Step 5: Update example ✓
- Added stage 7 (gap measurement) to `process_sample.rs`
- Prints gap table: feature pairs, distances, widths, areas, elevations

### Quality checks ✓
- `cargo fmt`: clean
- `cargo clippy -D warnings`: clean
- 48 unit tests + 12 integration tests pass (excluding 3 Powell Market real-scan tests that require the PLY file and take >60s in debug mode)

## Deviations from Plan

- **Ticket arithmetic correction:** The ticket says "two cylinders 10ft apart, 2ft
  diameter → gap width 6ft" but the formula `clear_width = dist - (spread/2 + spread/2)`
  gives `10 - (1+1) = 8ft`. Implemented the formula as specified in acceptance criteria;
  test uses 8ft as the expected value.

- **pt-features lib.rs:** Found broken workspace member (missing lib.rs). A pre-commit
  hook populated it automatically — not part of this ticket's scope.
