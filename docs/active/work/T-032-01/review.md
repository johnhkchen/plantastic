# T-032-01 Review: scan-cli-example

## Summary

Created a CLI example that runs a PLY file through the full pt-scan pipeline with per-stage timing and diagnostic metadata output.

## Files Changed

### Created
- `crates/pt-scan/examples/process_sample.rs` — CLI example (~170 lines)

### Modified
- `.gitignore` — Added `*.glb` and `*.png` rules for `assets/scans/samples/`
- `justfile` — Added `process-scan` recipe (release mode, configurable path)

## Acceptance Criteria Check

| Criterion | Status |
|---|---|
| `crates/pt-scan/examples/process_sample.rs` | ✓ Created |
| Accepts PLY path as CLI arg (default: sample scan) | ✓ `env::args().nth(1)` with fallback |
| Full pipeline: parse → downsample → outlier → RANSAC → mesh → export | ✓ All 5 stages |
| Prints timing for each stage and total | ✓ Per-stage + total |
| Prints metadata: counts, ground plane, obstacle height range | ✓ Plus bbox, extent |
| Writes `{name}-terrain.glb` and `{name}-planview.png` | ✓ Alongside input |
| Config tuned for outdoor urban scan | ✓ 5cm voxels, k=20, 5cm RANSAC threshold |
| `just process-scan path` recipe | ✓ Added |
| Performance target: <60s for 20M points | ⚠ Not validated — requires 294MB sample PLY (gitignored) |

## Design Decisions

1. **Direct sub-module calls** instead of `process_scan()` black box — enables per-stage timing (5 stages instead of 2).
2. **No new dependencies** — uses only stdlib for arg parsing, timing, file I/O.
3. **Release mode in justfile** — debug mode would be 10-20× slower, meaningless for benchmarking.

## Test Coverage

- No new tests. This is an example binary, not library code.
- All library functions used are already covered by existing unit + integration tests.
- Compilation verified: `cargo build -p pt-scan --examples` passes.
- Quality gate: `just check` passes with no regressions.

## Scenario Dashboard

No scenario changes expected — this ticket doesn't advance customer-facing capabilities. It's a developer diagnostic tool. The scenario dashboard was verified unchanged by `just check`.

## Open Concerns

1. **Performance target unverified.** The 294MB sample PLY is gitignored and not available in CI. The <60s target can only be validated manually with `just process-scan`. This is expected per the ticket ("The 294MB file is gitignored").

2. **Single-threaded pipeline.** All stages run sequentially on one thread. For 20M+ points, parallelizing voxel downsampling or k-NN could help if the 60s target is tight. Not needed now but worth noting for E-014.

3. **No `--config` overrides.** The outdoor urban config is hardcoded as constants. If we later need to tune for different scan types (indoor, residential), we'd add CLI flags. Not in scope for this ticket.
