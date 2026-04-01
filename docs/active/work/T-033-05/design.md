# T-033-05 Design: Rerun Debug Visualization

## Decision: Example binary with dev-dependency

### Approach chosen

Add `rerun` as a **dev-dependency** in pt-scan's Cargo.toml. Create a single example
binary `examples/debug_segmentation.rs` that runs the full pipeline and logs each stage
to Rerun with timeline stepping. Add a `just debug-scan` recipe.

### Why dev-dependency (not Cargo feature)

**Option A: dev-dependency** (chosen)
- `rerun` only compiles when running examples or tests, never for `cargo build --release`
- No Cargo feature ceremony — simpler Cargo.toml
- Example binaries are naturally excluded from production builds
- Matches existing pattern: `pt-features`, `geo`, `tokio` are already dev-deps

**Option B: Cargo feature `debug-viz`**
- More explicit gating but adds unnecessary complexity
- Risk of someone enabling the feature accidentally in workspace
- Examples already serve as the gating mechanism

**Option C: Separate crate `pt-scan-debug`**
- Over-engineering for a single example file
- Would need to re-export or depend on pt-scan internals

### Visualization stages (timeline stepping)

Use `set_time_sequence("stage", N)` to create temporal steps:

1. **"raw" (step 0)** — All points from PLY after processing, colored by RGB
   - Ground points logged at `/ground/points`
   - Obstacle points logged at `/obstacles/points`

2. **"features" (step 1)** — Obstacle points colored by eigenvalue features
   - `/features/planarity` — blue→red heatmap
   - `/features/linearity` — blue→red heatmap  
   - `/features/sphericity` — blue→red heatmap
   - Each logged as Points3D with per-point colors mapped from feature values

3. **"clustered" (step 2)** — Points colored by HDBSCAN cluster ID
   - `/clusters/points` — distinct color per cluster
   - `/clusters/noise` — gray points for noise (label -1)

4. **"candidates" (step 3)** — Feature candidates as 3D bounding boxes
   - `/candidates/boxes` — Boxes3D with half-sizes, colored per cluster, labeled with cluster_id + height/spread

### Color mapping strategy

- **RGB points**: Use actual point colors if present, white fallback
- **Feature heatmaps**: Linear interpolation blue (0.0) → red (1.0) for [0,1] features
  (planarity, linearity, sphericity are already in [0,1])
- **Cluster colors**: Deterministic palette from cluster ID (golden ratio hue spacing)
- **Noise**: Gray `[128, 128, 128]`
- **Bounding boxes**: Match cluster color, 50% alpha

### Entity path hierarchy

```
/ground/points              — ground after RANSAC
/obstacles/points           — obstacles after RANSAC
/features/planarity         — obstacle heatmap
/features/linearity         — obstacle heatmap
/features/sphericity        — obstacle heatmap
/clusters/assigned          — clustered obstacle points
/clusters/noise             — noise points
/candidates/boxes           — 3D bounding boxes with labels
```

### Justfile recipe

```
debug-scan path="assets/scans/samples/powell-market-downsampled.ply":
    cargo run -p pt-scan --example debug_segmentation --release -- "{{path}}"
```

Release mode for realistic processing speed. Rerun viewer spawns automatically.

### What was rejected

- **Logging to .rrd file**: Adds file management complexity. `rr::spawn()` opens the
  viewer directly — simpler for the debugging use case.
- **Interactive parameter tuning**: Out of scope. The user can change HdbscanConfig
  constants in the example and re-run. Real-time slider control would require a custom
  Rerun panel or egui integration.
- **Logging every pipeline stage** (downsample, outlier removal): Adds noise. The
  interesting stages for HDBSCAN tuning are raw → features → clustered → candidates.
