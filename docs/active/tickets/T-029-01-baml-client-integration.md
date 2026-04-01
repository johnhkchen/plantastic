---
id: T-029-01
story: S-029
title: baml-client-integration
type: task
status: open
priority: high
phase: ready
depends_on: []
---

## Context

BAML is initialized (baml_src/proposal.baml) and the Rust client is generated (baml_client/). This ticket integrates the generated code into the workspace so it compiles and is callable.

## Acceptance Criteria

- baml_client/ added to workspace or included as a module in pt-proposal
- `baml-runtime` dependency added to workspace Cargo.toml
- Generated types (ProposalContent, TierNarrative, ZoneCallout, TierInput, ZoneSummary) importable
- GenerateProposalNarrative function callable with async Rust
- `just check` passes (baml_client compiles without warnings)

## Implementation Notes

- The generated code uses `baml` crate derive macros — add `baml` to workspace deps
- baml_client is generated code checked into the repo (standard BAML pattern)
- The generator config (generators.baml) points output to `../baml_client`
- After any `.baml` file change: `baml-cli generate` to regenerate
- Add `baml_client/` to `.gitignore` comment explaining it's generated but committed
