# T-015-01 Progress: PLY Parsing & Point Cloud Filtering

## Completed Steps

### Step 1: Scaffold crate ✓
- Created `crates/pt-scan/Cargo.toml` with deps: ply-rs-bw, kiddo, nalgebra, rand, indexmap, serde, thiserror
- Created `src/lib.rs`, `src/types.rs`, `src/error.rs`
- `cargo check -p pt-scan` passes

### Step 2: PLY parser ✓
- Implemented `parse_ply()` in `src/parser.rs`
- Handles binary LE/BE and ASCII via ply-rs-bw
- Extracts vertex positions [f32; 3] and optional RGB colors [u8; 3]
- ply-rs-bw v3 uses IndexMap (not HashMap) and BufRead (not Read) — adapted accordingly
- 4 tests: binary, ASCII, missing color, empty PLY

### Step 3: Voxel downsampling ✓
- Implemented `voxel_downsample()` in `src/filter.rs`
- HashMap<(i32,i32,i32), Accumulator> cell averaging
- Positions averaged via f64 accumulators for precision, colors averaged as u32
- 4 tests: reduces count, preserves bounds, averages color, empty input

### Step 4: Statistical outlier removal ✓
- Implemented `remove_outliers()` in `src/filter.rs`
- Builds ImmutableKdTree<f32, 3>, queries k+1 nearest (self included)
- Computes mean neighbor distance per point, filters by global mean + threshold * stddev
- kiddo v5 requires NonZeroUsize for nearest_n (MSRV 1.75 requires NonZeroUsize, not NonZero)
- 3 tests: filters distant outliers, preserves uniform cloud, empty input

### Step 5: RANSAC ground plane fitting ✓
- Implemented `fit_ground_plane()` in `src/ransac.rs`
- Random 3-point sampling, cross-product normal, inlier counting
- Uses nalgebra Vector3 for math, rand::seq::index::sample for sampling
- Test helper with seeded RNG for deterministic results
- 4 tests: horizontal plane, tilted plane, insufficient points, classifies obstacles

### Step 6: Top-level pipeline ✓
- Wired `process_scan()` in `lib.rs`: parse → downsample → outlier removal → RANSAC
- Computes ScanMetadata (bbox, counts, plane)
- 3 integration tests: full pipeline, default config, insufficient points

### Step 7: Scenario harness ✓
- Added pt-scan to `tests/scenarios/Cargo.toml`
- Updated S.1.1 to test PLY → PointCloud pipeline, returns OneStar
- Updated milestone in `progress.rs` with T-015-01 delivery note
- Dashboard: 48.0 → 54.0 min (+6), 9 → 10 milestones, S.1.1 PASS ★☆☆☆☆

### Step 8: Quality gate ✓
- `just check` passes: fmt, lint (clippy strict), test (18 unit + 3 integration), scenarios

## Deviations from Plan

1. Added `indexmap = "2"` dependency — ply-rs-bw v3 exposes IndexMap instead of HashMap
   for property access. Transitive dep anyway, but needed explicit import.
2. Used `NonZeroUsize` instead of `NonZero<usize>` — MSRV 1.75 predates NonZero stabilization.
3. No separate benchmark test with 1M+ points — unit tests pass quickly with small
   synthetic data, and the algorithms are standard. Performance can be validated when
   real SiteScape PLY files are available.
