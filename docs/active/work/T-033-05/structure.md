# T-033-05 Structure: Rerun Debug Visualization

## Files modified

### `crates/pt-scan/Cargo.toml`

Add to `[dev-dependencies]`:
```toml
rerun = "0.22"
```

### `crates/pt-scan/examples/debug_segmentation.rs` (new)

Single example binary, ~200 lines. Structure:

```
main()
├── parse CLI args (PLY path, optional min_cluster_size override)
├── process_scan_timed() — parse + downsample + outlier + RANSAC
├── init Rerun recording stream via rr::spawn()
├── log_raw_points() — stage 0
│   ├── log ground points at /ground/points with RGB colors
│   └── log obstacle points at /obstacles/points with RGB colors
├── compute_point_features() — k=30 neighbors
├── log_feature_heatmaps() — stage 1
│   ├── log /features/planarity — blue→red per-point coloring
│   ├── log /features/linearity — blue→red per-point coloring
│   └── log /features/sphericity — blue→red per-point coloring
├── hdbscan_cluster() — 6D augmented clustering
├── log_clusters() — stage 2
│   ├── log /clusters/assigned — colored by cluster ID
│   └── log /clusters/noise — gray
├── extract_candidates()
├── log_candidates() — stage 3
│   └── log /candidates/boxes — Boxes3D with labels
└── print summary table (cluster count, noise count, timing)
```

Helper functions (module-local):
- `feature_to_color(value: f32) -> [u8; 3]` — linear blue→red for [0,1]
- `cluster_color(id: u32) -> [u8; 3]` — golden ratio hue spacing
- `point_color(point: &Point) -> [u8; 3]` — RGB or white fallback

### `justfile`

Add recipe under "Scan Processing" section:
```just
# Debug segmentation with Rerun 3D viewer (opens viewer automatically)
debug-scan path="assets/scans/samples/powell-market-downsampled.ply":
    cargo run -p pt-scan --example debug_segmentation --release -- "{{path}}"
```

## Module boundaries

- No new modules in pt-scan's `src/` — this is purely an example binary
- No new public API — all needed types are already exported from `lib.rs`
- Rerun is dev-only — never appears in production dependency tree

## Ordering

1. Add `rerun` dev-dependency to Cargo.toml
2. Create example binary
3. Add justfile recipe
4. Verify with `cargo build -p pt-scan --examples` and `just lint`
