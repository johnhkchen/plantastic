# T-033-05 Progress: Rerun Debug Visualization

## Completed

1. **Added `rerun = "0.22"` dev-dependency** to `crates/pt-scan/Cargo.toml`
   - Compiles cleanly, ~50 MB dep isolated to dev/example builds only

2. **Created `examples/debug_segmentation.rs`** (~230 lines)
   - Loads PLY through full pipeline (`process_scan_timed`)
   - Spawns Rerun viewer via `RecordingStream::spawn()`
   - Logs 4 timeline stages:
     - Stage 0: raw ground + obstacle points with RGB colors
     - Stage 1: eigenvalue feature heatmaps (planarity, linearity, sphericity)
     - Stage 2: HDBSCAN cluster assignments (colored by cluster ID) + noise (gray)
     - Stage 3: feature candidate bounding boxes with labels
   - Clean clippy with `#[allow]` for intentional float→int color casts
   - HSV color generation for distinct cluster colors (golden ratio hue spacing)

3. **Added `just debug-scan` recipe** to justfile
   - Runs example in release mode with configurable PLY path
   - Default: powell-market-downsampled.ply

4. **Quality checks**
   - `just fmt-check` — PASS
   - `just lint` — PASS
   - `just test` — pre-existing SIGKILL on powell_market_hdbscan (OOM in debug mode, not related to this ticket)
   - `just scenarios` — 87.5 min (no regression from baseline)

## No deviations from plan
