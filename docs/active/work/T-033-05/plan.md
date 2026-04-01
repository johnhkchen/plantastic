# T-033-05 Plan: Rerun Debug Visualization

## Step 1: Add rerun dev-dependency

- Edit `crates/pt-scan/Cargo.toml` to add `rerun = "0.22"` under `[dev-dependencies]`
- Run `cargo check -p pt-scan` to verify dependency resolution
- **Verification**: `cargo metadata` shows rerun as a dev-dependency of pt-scan

## Step 2: Create debug_segmentation.rs example

Create `crates/pt-scan/examples/debug_segmentation.rs` with:

1. CLI arg parsing: PLY path (default: powell-market-downsampled.ply)
2. Pipeline execution: `process_scan_timed()` with standard config
3. Rerun initialization: `RecordingStream::spawn("debug_segmentation", Default::default())`
4. Stage 0 (raw): Log ground + obstacle points with RGB colors using `rr::Points3D`
5. Stage 1 (features): Compute eigenvalue features, log 3 heatmaps (planarity, linearity, sphericity)
6. Stage 2 (clustered): Run HDBSCAN, log points colored by cluster ID, noise in gray
7. Stage 3 (candidates): Extract candidates, log bounding boxes with labels
8. Summary output: print cluster stats to terminal

**Verification**: `cargo build -p pt-scan --example debug_segmentation`

## Step 3: Add justfile recipe

Add `debug-scan` recipe to justfile under "Scan Processing" section.

**Verification**: `just --list` shows `debug-scan`

## Step 4: Lint and format

- `just fmt`
- `just lint`
- Fix any clippy warnings

**Verification**: `just fmt-check && just lint`

## Step 5: Verify tests still pass

- `just test` — ensure no regressions from adding the dev-dependency
- `just scenarios` — verify scenario dashboard unchanged

**Verification**: All tests pass, scenarios at 87.5 min (no regression)

## Testing strategy

This ticket is a dev-only debug tool — no new unit or integration tests needed.
The example binary is verified by compilation (`cargo build --examples`).
The acceptance test is visual: run `just debug-scan` and confirm the Rerun viewer
shows all 4 stages with correct coloring.

No scenarios are expected to change — this is tooling infrastructure.
