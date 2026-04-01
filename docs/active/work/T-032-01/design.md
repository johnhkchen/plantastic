# T-032-01 Design: scan-cli-example

## Decision: Call sub-modules directly for per-stage timing

### Option A: Black-box `process_scan` + `generate_terrain`

- Only 2 timing measurements (process + terrain)
- Simpler code, uses public API as documented
- Insufficient: ticket requires "timing for each stage"

### Option B: Call sub-modules directly for per-stage timing

- 6 timing measurements (parse, downsample, outlier, RANSAC, terrain gen, file write)
- Uses pub module functions (parser::parse_ply, filter::*, ransac::*)
- More informative, matches ticket's "prints timing for each stage" requirement
- Slightly more code but still ~150 lines

**Chosen: Option B.** The whole point of this example is to measure pipeline performance. Two timing buckets defeat the purpose.

## Pipeline Implementation

```
1. Parse args (PLY path, optional config overrides)
2. Open file → BufReader
3. Stage timings:
   a. parse_ply(reader) → Vec<Point>          — print count
   b. voxel_downsample(&points, 0.05) → Vec   — print count
   c. remove_outliers(&ds, 20, 2.0) → Vec     — print count
   d. fit_ground_plane(&filtered, 1000, 0.05)  — print ground/obstacle split
   e. generate_terrain(&cloud, &config)        — print triangle count
   f. write GLB + PNG files                    — print file sizes
4. Print summary: total time, all metadata
```

## Config: Outdoor Urban Preset

Per ticket: `ScanConfig { voxel_size: 0.05, outlier_k: 20, outlier_threshold: 2.0, ransac_iterations: 1000, ransac_threshold: 0.05 }`

Key differences from default:
- voxel_size: 0.05 vs 0.02 (2.5× larger cells → ~6× fewer points → much faster)
- outlier_k: 20 vs 30 (fewer neighbors → faster k-NN)
- ransac_threshold: 0.05 vs 0.02 (outdoor terrain is rougher)

## Output File Naming

Input: `/path/to/Scan at 09.23.ply`
Outputs:
- `/path/to/Scan at 09.23-terrain.glb`
- `/path/to/Scan at 09.23-planview.png`

Use `Path::with_extension` won't work (replaces .ply). Instead, strip `.ply` suffix and append.

## CLI Interface

```
cargo run -p pt-scan --example process_sample [--release] -- [path]
```

Default path: `assets/scans/samples/Scan at 09.23.ply`

No clap/structopt dependency — just `std::env::args()`. This is a diagnostic example, not a user-facing tool.

## Justfile Recipe

```just
# Process a PLY scan through the full pipeline (release mode for realistic timing)
process-scan path="assets/scans/samples/Scan at 09.23.ply":
    cargo run -p pt-scan --example process_sample --release -- "{{path}}"
```

Release mode is important — debug mode will be 10-20× slower and miss the <60s target.

## Print Format

Clean, scannable output with aligned sections:

```
── Scan Processing Pipeline ──────────────────
Input: assets/scans/samples/Scan at 09.23.ply (294.1 MB)

[1/5] Parse PLY .............. 3.2s   20,543,210 points
[2/5] Voxel downsample ...... 1.1s      612,340 points (5.0cm voxels)
[3/5] Outlier removal ....... 8.4s      598,120 points (k=20, σ=2.0)
[4/5] RANSAC ground fit ..... 0.3s      ground: 482,100 | obstacles: 116,020
[5/5] Terrain generation .... 4.2s      48,320 triangles

── Metadata ──────────────────────────────────
Bounding box: [-12.3, -8.1, -0.5] → [15.2, 22.4, 8.3]
Extent: 27.5 × 30.5 × 8.8 m
Ground plane: normal=[0.01, -0.02, 1.00] d=-0.15
Obstacle height range: 0.1 – 8.3 m

── Output ────────────────────────────────────
Terrain: Scan at 09.23-terrain.glb (2.1 MB)
Plan view: Scan at 09.23-planview.png (340 KB)
Total: 17.2s
```

## Rejected Alternatives

1. **Add clap dependency** — Overkill for a single-arg example. Would add compile time.
2. **Parallel stages** — Stages are sequential by nature (each depends on previous output).
3. **Progress bars** — Nice but adds indicatif dependency. Print statements are sufficient.
4. **JSON output mode** — Could be useful later but not in acceptance criteria.
