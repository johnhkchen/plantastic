# Research: T-029-01 BAML Client Integration

## Current State

### BAML Source Files (`baml_src/`)
Three files define the BAML schema:
- **`clients.baml`** — Two Anthropic clients (Sonnet for quality, Haiku for speed) with a fallback strategy and exponential retry.
- **`generators.baml`** — Rust output, version `0.218.0`, async mode, output dir `../` (relative to baml_src/ = workspace root).
- **`proposal.baml`** — Single function `GenerateProposalNarrative` with 5 classes: `ProposalContent`, `TierNarrative`, `ZoneCallout`, `TierInput`, `ZoneSummary`.

### Generated Client (`baml_client/`)
Fully generated Rust module at workspace root. Structure:
```
baml_client/
  mod.rs              — re-exports, defines B (async client singleton)
  runtime.rs          — BamlRuntime init via OnceLock, FunctionOptions builder
  baml_source_map.rs  — embedded .baml sources for runtime
  functions/
    async_client.rs   — GenerateProposalNarrative struct with call/stream/parse
    sync_client.rs    — sync variant
  types/
    classes.rs        — 5 structs with BamlEncode/BamlDecode derives
  stream_types/       — partial streaming variants
  type_builder/       — dynamic type construction
```

Key observations:
- Generated code imports `baml::*` (e.g., `BamlRuntime`, `BamlEncode`, `BamlDecode`, `BamlError`, `AsyncStreamingCall`, `Collector`, `CancellationToken`, `ClientRegistry`).
- Module path is `crate::baml_client::*` — the generated mod.rs assumes it's a child module of some crate root.
- `#![allow(non_snake_case, unused_imports, non_camel_case_types, dead_code)]` suppresses lint warnings on generated code.
- `baml_source_map.rs` uses `crate::baml_client::` path references.

### Workspace Configuration (`Cargo.toml`)
- Members: `crates/*` and `tests/scenarios`
- **No `baml` dependency anywhere** — not in workspace deps, not in any crate Cargo.toml, not in Cargo.lock.
- Existing crates: plantastic-api, pt-climate, pt-geo, pt-materials, pt-project, pt-quote, pt-repo, pt-satellite, pt-scan, pt-solar, pt-test-utils.
- No `pt-proposal` crate exists yet.

### Dependency Chain
T-029-02 (depends on this ticket) will create:
- `ProposalNarrativeGenerator` trait in pt-proposal
- `BamlProposalGenerator` (real impl calling baml_client)
- `MockProposalGenerator` for tests

This means T-029-01 must make the generated types and function importable, but T-029-02 handles the trait abstraction.

## Key Constraints

1. **Generated code is checked in** — standard BAML pattern. Output dir in generators.baml is `../` from baml_src/, placing baml_client/ at workspace root.

2. **Module path assumption** — Generated files use `crate::baml_client::` internally. The mod.rs must be included via `mod baml_client;` in whatever crate's lib.rs hosts it.

3. **Workspace lints** — Clippy strict (warnings=errors). Generated code has `#![allow(...)]` but only at the mod.rs level. Clippy may still fire on generated patterns if workspace lints override inner attributes.

4. **baml crate version** — generators.baml specifies `0.218.0`. The `baml` crate on crates.io must match this version.

5. **No pt-proposal crate yet** — The ticket says "baml_client/ added to workspace or included as a module in pt-proposal." Either approach is valid.

6. **Async runtime** — Generated client is async. Workspace already uses `tokio` with `rt-multi-thread`.

## Patterns in Existing Crates

All crates follow the same pattern:
- `crates/{name}/Cargo.toml` with `edition.workspace = true`, `license.workspace = true`, `rust-version.workspace = true`
- `[lints] workspace = true`
- Internal deps via `path = "../pt-foo"`
- Workspace deps via `dep.workspace = true`

plantastic-api depends on domain crates (pt-quote, pt-project, etc.) and will eventually depend on pt-proposal (per T-029-02).

## Open Questions

1. Should baml_client/ stay at workspace root or move into a crate? Generator output dir would need changing if moved.
2. The `baml` crate version `0.218.0` — need to verify this exists on crates.io and what its actual dependency tree looks like (it pulls in tokio, serde, etc.).
3. Lint suppression: will `[lints] workspace = true` in a new crate conflict with the generated `#![allow(...)]`? Need to verify.
