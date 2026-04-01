# T-036-01 Plan: Measure Feature Gaps

## Step 1: Create `gap.rs` with types and core function

**Files:** `crates/pt-scan/src/gap.rs` (new)

Create the module with:
- `Gap` struct (8 fields, derive Debug/Clone/Serialize)
- `GapConfig` struct with Default impl (max_distance_ft = 30.0)
- `measure_gaps()` function implementing pairwise gap computation
- Private helpers: `centroid_distance_2d_ft()`, `ground_elevation_ft()`
- `M_TO_FT` constant

**Verify:** Compiles in isolation (checked by step 2).

## Step 2: Register module in lib.rs

**Files:** `crates/pt-scan/src/lib.rs` (modify)

Add `pub mod gap;` and `pub use gap::{measure_gaps, Gap, GapConfig};`

**Verify:** `cargo check -p pt-scan`

## Step 3: Unit tests in gap.rs

**Files:** `crates/pt-scan/src/gap.rs` (add tests module)

Tests:
1. **test_two_features_known_gap:** Two candidates with centroids 3.048m apart
   (≈10ft), spread_ft=2.0 each. Expected: 1 gap, clear_width=8.0ft,
   centroid_distance≈10.0ft.
   Independent arithmetic: 3.048m × 3.28084 = 10.0ft. 10 - (1+1) = 8ft.

2. **test_overlapping_features:** Centroids 0.3m apart, spread_ft=4.0ft each.
   centroid_dist ≈ 0.984ft. clear_width = 0.984 - (2+2) = -3.016 → filtered out.
   Expected: 0 gaps.

3. **test_beyond_threshold:** Centroids 20m apart (≈65.6ft) with default 30ft
   threshold. Expected: 0 gaps.

4. **test_empty_and_single:** 0 candidates → 0 gaps. 1 candidate → 0 gaps.

5. **test_ground_elevation:** Horizontal ground plane (normal=[0,0,1], d=-1.0).
   Elevation at any XY point = 1.0m = 3.28084ft.

6. **test_three_features_pairwise:** Three features in a line, 10ft apart.
   Two adjacent gaps (10ft), one far gap (20ft). With 30ft threshold, all 3 pairs.

All tests use `pt_test_utils::timed`.

**Verify:** `cargo test -p pt-scan -- gap`

## Step 4: Integration test

**Files:** `crates/pt-scan/tests/integration.rs` (modify)

Add test that builds a synthetic PLY with two obstacle clusters separated by a
known distance, runs the full pipeline (parse → cluster → candidates → gaps),
and verifies the gap dimensions.

**Verify:** `cargo test -p pt-scan --test integration`

## Step 5: Update example

**Files:** `crates/pt-scan/examples/process_sample.rs` (modify)

After the feature candidates table, add:
- Gap measurement using `GapConfig::default()`
- Print gap table: feature_a → feature_b, distance, clear_width, area

**Verify:** `cargo build --example process_sample -p pt-scan`

## Step 6: Quality gate

**Verify:** `just check` (format + lint + test + scenarios)

Fix any clippy warnings or formatting issues.

## Testing Strategy

- **Unit tests (step 3):** Cover all branches — normal gap, overlapping, threshold
  filter, empty input, elevation computation, multi-feature pairwise.
  All expected values computed independently (not using system functions).
- **Integration test (step 4):** End-to-end from PLY bytes to gap measurements.
  Verifies the pipeline wiring, not just the gap math.
- **No mocks needed:** Pure geometry — no external services, no I/O.
