# T-036-03 Progress: Scan-to-Quote Demo

## Completed

- [x] Step 1: Added dev-dependencies to pt-scan/Cargo.toml (pt-quote, pt-project, pt-materials, rust_decimal, geo)
- [x] Step 2: Created `crates/pt-scan/examples/scan_to_quote.rs` — full pipeline example
- [x] Step 3: Added `just scan-to-quote` recipe to Justfile
- [x] Step 4: Quality gates
  - [x] `cargo fmt` — formatted
  - [x] `cargo clippy --workspace --all-targets -- -D warnings` — clean
  - [x] Library tests pass (`cargo test --workspace --lib`)
  - [x] Scenarios: 10 pass, 0 fail, no regressions (87.5/240 min unchanged)

## Deviations from Plan

1. **Gap selection**: Changed from "closest gap" to "largest area gap" — the closest
   gap was only 1.0 ft wide / 7 sqft, too small for a meaningful planter demo. Largest
   area gap gives ~109 sqft, much more representative.

2. **Plant spacing adjusted**: Originally used 4"/6" spacing matching the ticket's
   aspirational example. Adjusted to 8"/12"/6" for more realistic numbers at the scan's
   actual gap size (~109 sqft). Pricing adjusted accordingly ($5/$8/$3 per plant).

3. **No tree trunks in scan**: The Powell & Market downsampled PLY produces 14 features,
   all classified as hardscape/structure by the mock classifier (short, flat elements).
   The ticket's "2 tree trunks" was aspirational. The demo works with whatever features
   the scan contains. With `--live` mode and real LLM classification, results would
   likely be more nuanced.

## Pre-existing Issue

The Powell & Market integration tests (`test_powell_market_*`) timeout in debug mode
due to processing 120K+ points. This is not caused by this ticket's changes — we only
added an example binary and dev-dependencies. No library code was modified.
