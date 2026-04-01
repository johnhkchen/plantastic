# Structure: T-029-01 BAML Client Integration

## Files Created

### `crates/pt-proposal/Cargo.toml`
New crate manifest following workspace conventions.
- Dependencies: `baml.workspace = true`, `serde.workspace = true`, `tokio.workspace = true`
- Dev-dependencies: `pt-test-utils = { path = "../pt-test-utils" }`
- `[lints] workspace = true`

### `crates/pt-proposal/src/lib.rs`
Crate root. Includes baml_client via `#[path]` and re-exports public types.

```
lib.rs
├── mod baml_client (via #[path = "../../../baml_client"])
│   └── (all generated code)
└── pub re-exports
    ├── types::{ProposalContent, TierNarrative, ZoneCallout, TierInput, ZoneSummary}
    ├── B (async client)
    └── Error (BamlError alias)
```

## Files Modified

### `Cargo.toml` (workspace root)
Add to `[workspace.dependencies]`:
```toml
baml = "0.218.0"
```

No changes to `[workspace.members]` — the `crates/*` glob automatically picks up pt-proposal.

## Files Unchanged

### `baml_client/` (workspace root)
No changes. Generated code stays in place. The `#[path]` attribute in lib.rs reaches it.

### `baml_src/` (workspace root)
No changes to BAML definitions or generator config.

## Module Boundaries

### Public Interface of pt-proposal

```rust
// Types (re-exported from baml_client::types)
pub use baml_client::types::{
    ProposalContent,
    TierNarrative,
    ZoneCallout,
    TierInput,
    ZoneSummary,
};

// Async client singleton
pub use baml_client::B;

// Function struct for direct use
pub use baml_client::async_client::GenerateProposalNarrative;

// Error type
pub type Error = baml::BamlError;
```

### Internal (not re-exported)
- `baml_client::runtime` — runtime init, FunctionOptions
- `baml_client::stream_types` — partial streaming types
- `baml_client::type_builder` — dynamic type construction
- `baml_client::baml_source_map` — embedded BAML sources

## Lint Strategy

The generated baml_client code uses patterns that conflict with workspace clippy rules (non_snake_case fields like `GenerateProposalNarrative`, dead_code, etc.). The generated mod.rs already has `#![allow(...)]` but that's an inner attribute which may not suppress workspace-level deny rules.

Strategy: Wrap the `#[path]` include with `#[allow(clippy::all)]` at the module declaration to blanket-suppress clippy on generated code:

```rust
#[allow(clippy::all, non_snake_case, unused_imports, non_camel_case_types, dead_code)]
#[path = "../../../baml_client"]
mod baml_client;
```

## Ordering

1. Add `baml` to workspace deps (Cargo.toml)
2. Create crate directory and Cargo.toml
3. Create lib.rs with path include and re-exports
4. Verify compilation
5. Run `just check`
