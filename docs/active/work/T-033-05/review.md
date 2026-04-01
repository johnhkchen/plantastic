# T-033-05 Review: Rerun Debug Visualization

## Summary

Added Rerun-based 3D debug visualization for the pt-scan segmentation pipeline.
This is a dev-only debugging tool for tuning HDBSCAN parameters and inspecting
eigenvalue features — never shipped in production.

## Files changed

### Modified
- `crates/pt-scan/Cargo.toml` — added `rerun = "0.22"` to `[dev-dependencies]`
- `justfile` — added `debug-scan` recipe

### Created
- `crates/pt-scan/examples/debug_segmentation.rs` — ~230 lines, Rerun debug example

## Acceptance criteria verification

| Criterion | Status |
|-----------|--------|
| `rerun` as dev-dependency in pt-scan | DONE |
| `examples/debug_segmentation.rs` created | DONE |
| Load pre-downsampled PLY | DONE (via `process_scan_timed`) |
| Log raw points colored by RGB | DONE (stage 0: ground + obstacles) |
| Log eigenvalue feature heatmaps | DONE (stage 1: planarity, linearity, sphericity) |
| Log HDBSCAN cluster assignments | DONE (stage 2: colored by cluster ID) |
| Log noise points separately (gray) | DONE (stage 2: gray, smaller radius) |
| Log feature candidates as 3D bounding boxes with labels | DONE (stage 3: Boxes3D) |
| `just debug-scan <path>` recipe | DONE |
| NOT a production dependency | DONE (dev-dependency only) |

## Test coverage

This ticket is a dev-only debugging tool with no new production code. Coverage:
- **Compilation**: verified via `cargo check --example debug_segmentation`
- **Lint**: clean clippy pass with strict warnings
- **No new tests needed**: example binary is verified by compilation; visual verification
  requires running Rerun viewer interactively

## Scenario dashboard

- **Before**: 87.5 min / 240.0 min (36.5%)
- **After**: 87.5 min / 240.0 min (36.5%)
- No change expected — this is dev tooling, not customer-facing capability

## Open concerns

1. **Pre-existing test OOM**: `test_powell_market_hdbscan` (from T-033-04) SIGKILLs in
   debug mode, likely due to HDBSCAN on the full PLY being memory-intensive without
   `--release` optimizations. Not caused by this ticket — the test was already failing
   before these changes. Should be addressed by T-033-04 or a follow-up ticket (either
   skip in debug mode or reduce test data size).

2. **Rerun version**: Pinned to `0.22` per ticket spec. Latest is `0.31.1` — could
   upgrade later but the 0.22 API is stable and sufficient.

3. **Visual verification**: The example compiles and runs but the actual Rerun viewer
   output needs manual visual inspection to confirm all stages render correctly. This
   is inherent to visualization tooling.
