# T-036-03 Plan: Scan-to-Quote Demo

## Step 1: Add dev-dependencies to pt-scan/Cargo.toml

Add the crates needed by the example: pt-features, pt-quote, pt-project, pt-materials,
rust_decimal, geo, tokio.

**Verify:** `cargo check -p pt-scan --example scan_to_quote` compiles (will fail until
step 2, but deps resolve).

## Step 2: Create scan_to_quote.rs example

Write `crates/pt-scan/examples/scan_to_quote.rs` with the full pipeline:

1. Parse CLI args (path, optional --live)
2. Load and process PLY scan
3. Cluster obstacles, extract feature candidates
4. Classify features (mock by default)
5. Measure gaps between features
6. Convert best gap to a Zone polygon
7. Build three material tiers (Good/Better/Best)
8. Compute quotes via pt-quote for each tier
9. Print investor-ready summary
10. Write terrain GLB and plan-view PNG

**Verify:** `cargo run -p pt-scan --example scan_to_quote --release -- "assets/scans/samples/powell-market-downsampled.ply"` produces quote output.

## Step 3: Add Justfile recipe

Add `scan-to-quote` recipe pointing to the example.

**Verify:** `just scan-to-quote` runs end-to-end.

## Step 4: Run quality gates

- `just fmt` — format the new code
- `just lint` — ensure clippy passes
- `just test` — all existing tests still pass
- `just scenarios` — no regressions

**Verify:** All four gates pass.

## Testing Strategy

This is a stitching ticket — all components are already tested:
- pt-scan: process_scan, clustering, gap measurement (unit + integration tests)
- pt-features: mock classifier (unit tests)
- pt-quote: compute_quote (10+ unit tests covering all unit types)
- Scenarios S.1.1, S.3.1, S.3.2 validate the underlying pipelines

The example itself is verified by running it:
- Deterministic output (same scan → same dollar amounts every run)
- Format matches ticket spec
- Terrain GLB written successfully
- Completes in < 10s with mock generators

No new unit tests are needed — the example is the integration test.

## Commit Strategy

Single commit: "T-036-03: scan-to-quote demo — full pipeline from PLY to three-tier quote"

Contains:
- New file: crates/pt-scan/examples/scan_to_quote.rs
- Modified: crates/pt-scan/Cargo.toml (dev-deps)
- Modified: Justfile (new recipe)

## Risk Assessment

**Low risk.** No library code changes. All new code is in an example binary that doesn't
affect any existing functionality. The only modifications to existing files are additive
(new deps, new recipe).

**Known constraint:** The actual gap dimensions and dollar amounts depend on the scan
data. The demo output will show whatever the Powell & Market scan produces — we format
it nicely but don't hardcode the numbers.
