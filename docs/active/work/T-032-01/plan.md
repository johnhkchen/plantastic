# T-032-01 Plan: scan-cli-example

## Steps

### Step 1: Verify sub-module function signatures

Read parser.rs, filter.rs, ransac.rs to confirm exact pub fn signatures match what the example will call. Check that GroundClassification is accessible (may need to import from types or ransac).

### Step 2: Check gitignore coverage

Verify that .glb and .png outputs in assets/scans/samples/ are covered by existing gitignore rules. Add rules if needed.

### Step 3: Create `crates/pt-scan/examples/process_sample.rs`

Write the example with:
- CLI arg parsing (default path to sample PLY)
- Per-stage timing using `std::time::Instant`
- Direct calls to parser, filter, ransac sub-modules
- PointCloud assembly from classification results
- Terrain generation via `generate_terrain`
- File output (GLB + PNG) alongside input
- Formatted output with counts, timings, metadata

### Step 4: Add justfile recipe

Add `process-scan` recipe that runs the example in release mode with a configurable path argument.

### Step 5: Build and lint check

Run `cargo build -p pt-scan --examples` to verify compilation.
Run `just lint` to verify clippy passes.
Run `just fmt` to ensure formatting.

### Step 6: Run `just check`

Full quality gate: fmt-check + lint + test + scenarios. No library code changed, so this should pass cleanly.

## Testing Strategy

- **No new tests needed.** This is an example binary, not library code. All library functions are already tested.
- **Compilation test:** `cargo build -p pt-scan --examples` proves the example compiles and the API is used correctly.
- **Manual validation:** Run `just process-scan` against the 294MB sample PLY (not in CI — file is gitignored).
- **Quality gate:** `just check` ensures no regressions.

## Commit Plan

Single commit: "T-032-01: scan CLI example with per-stage timing + justfile recipe"

Contains:
- `crates/pt-scan/examples/process_sample.rs` (new)
- `justfile` (modified — new recipe)
- `.gitignore` (modified if needed)
