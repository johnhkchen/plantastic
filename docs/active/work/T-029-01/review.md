# Review: T-029-01 BAML Client Integration

## Summary

Integrated the BAML-generated Rust client into the workspace by creating a new `pt-proposal` crate that includes `baml_client/` via `#[path]` and re-exports public types and the async client.

## Files Changed

### Created
- **`crates/pt-proposal/Cargo.toml`** — New crate with `baml`, `serde`, `tokio` workspace deps.
- **`crates/pt-proposal/src/lib.rs`** — Includes `baml_client/mod.rs` via `#[path]`, re-exports types (`ProposalContent`, `TierInput`, `TierNarrative`, `ZoneCallout`, `ZoneSummary`), async client (`B`, `GenerateProposalNarrative`), and `Error` alias.

### Modified
- **`Cargo.toml`** (workspace root) — Added `baml = "0.218.0"` to `[workspace.dependencies]`.
- **`baml_client/**`** (multiple files) — Reformatted by `cargo fmt`. No semantic changes. Files: `mod.rs`, `runtime.rs`, `baml_source_map.rs`, `functions/async_client.rs`, `functions/sync_client.rs`, `functions/mod.rs`, `types/classes.rs`, `types/mod.rs`, `stream_types/classes.rs`, `type_builder/mod.rs`, `type_builder/classes.rs`, `type_builder/enums.rs`.

## Acceptance Criteria Verification

| Criterion | Status |
|---|---|
| baml_client/ added to workspace or included as module in pt-proposal | Done — `#[path]` include in pt-proposal |
| `baml-runtime` dependency added to workspace Cargo.toml | Done — `baml = "0.218.0"` (baml crate includes runtime) |
| Generated types importable | Done — `use pt_proposal::{ProposalContent, TierInput, ...}` works |
| GenerateProposalNarrative function callable with async Rust | Done — `pt_proposal::B.GenerateProposalNarrative.call(...)` compiles |
| `just check` passes | Done — all four gates green |

## Test Coverage

No new tests added. This ticket is purely structural — it makes generated code compile within the workspace. Testing that BAML's generated encode/decode works is testing the BAML framework, not Plantastic domain logic.

The real test coverage comes in T-029-02 which creates:
- `ProposalNarrativeGenerator` trait
- `MockProposalGenerator` for deterministic testing
- Integration with `AppState` for dependency injection

## Scenario Dashboard

- **Before:** 44.5 / 240.0 min (18.5%), 8 pass, 15/24 milestones
- **After:** 44.5 / 240.0 min (18.5%), 8 pass, 15/24 milestones
- **Delta:** No change (expected — this ticket is infrastructure for T-029-02)

No milestone claimed. The BAML client integration is a prerequisite for the "BAML AI layer" milestone listed in the dashboard, but the milestone should be claimed when the trait abstraction (T-029-02) and actual proposal generation are wired end-to-end.

## Open Concerns

1. **Generated code formatting**: `cargo fmt` reformatted all generated files. If `baml-cli generate` is run again, it will produce unformatted code, and `cargo fmt` must be run afterwards. Consider adding `cargo fmt` to the generator's `on_generate` hook in `generators.baml` (currently `echo done`).

2. **BAML version pinning**: The `baml` crate is pinned to `0.218.0` matching `generators.baml`. If the BAML CLI is upgraded, both must be updated together. This is standard BAML practice but worth noting.

3. **Generated code in git**: The reformatted baml_client/ files create a large diff. Future `baml-cli generate` runs will need `cargo fmt` to avoid format check failures.

4. **Lint suppression breadth**: The `#[allow(clippy::pedantic)]` on the baml_client module is broad. This is acceptable for generated code but means any hand-written code added to pt-proposal's lib.rs won't get pedantic checks on items inside the baml_client module scope.
