---
id: S-029
epic: E-013
title: BAML Proposal Narrative
status: open
priority: high
tickets: [T-029-01, T-029-02]
---

## Goal

Wire BAML into the Rust backend for generating proposal narrative text. The schema is defined (baml_src/proposal.baml), the Rust client is generated (baml_client/). This story adds the integration layer and mock infrastructure.

## Acceptance Criteria

- baml_client module compiles as part of a crate (pt-proposal or plantastic-api)
- GenerateProposalNarrative callable from Rust with typed input/output
- MockProposalGenerator trait returns canned ProposalContent for tests
- All integration tests use the mock — zero real LLM calls
- BAML playground test (sample_proposal) documented for manual verification
