# T-038-02 Progress: Fix Tenant Isolation Scenario

## Completed

- [x] Research: identified root cause (zone_type casing mismatch)
- [x] Design: decided to fix test payloads, not loosen API validation
- [x] Structure: single file change in infrastructure.rs
- [x] Plan: 4 steps — fix casing, add body logging to S.INFRA.2, add body logging to S.INFRA.1, verify
- [x] Fix zone_type casing: `"Patio"` → `"patio"`, `"Bed"` → `"bed"` in S.INFRA.2
- [x] Add response body to all failure messages in S.INFRA.2 (7 steps)
- [x] Add response body to all failure messages in S.INFRA.1 (9 steps)
- [x] Capture body variable where previously discarded with `_` (steps 6/8/9 in S.INFRA.1, steps 2/5/6/7 in S.INFRA.2)
- [x] `just fmt` — clean
- [x] `cargo check -p pt-scenarios` — compiles with no warnings
- [x] `cargo test -p pt-scenarios` — 2 passed, 0 failed
- [x] Scenario dashboard: 87.5 min / 240.0 min (unchanged from baseline)

## Deviations from Plan

None. Pre-existing lint errors in pt-scan (from T-033-03/04 in-progress work) prevent
`just lint` from passing workspace-wide, but pt-scenarios itself is clean.

## Remaining

- [x] Write review.md
