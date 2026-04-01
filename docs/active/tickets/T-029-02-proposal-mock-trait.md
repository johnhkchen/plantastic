---
id: T-029-02
story: S-029
title: proposal-mock-trait
type: task
status: open
priority: high
phase: done
depends_on: [T-029-01]
---

## Context

Integration tests must not call real LLMs (costs tokens, non-deterministic). This ticket creates a trait abstraction for proposal narrative generation with a mock implementation.

## Acceptance Criteria

- `ProposalNarrativeGenerator` trait in pt-proposal:
  ```rust
  #[async_trait]
  trait ProposalNarrativeGenerator {
      async fn generate(&self, input: ProposalInput) -> Result<ProposalContent, ProposalError>;
  }
  ```
- `BamlProposalGenerator` — real implementation calling BAML client
- `MockProposalGenerator` — returns canned ProposalContent with realistic text
- Mock content is deterministic: same input → same output (no randomness)
- All scenario and integration tests use `MockProposalGenerator`
- API route accepts the generator via `AppState` (dependency injection)
- Document how to test with real LLM: `ANTHROPIC_API_KEY=... just test-smoke`

## Implementation Notes

- The mock should return plausible narrative text (not "mock mock mock") so screenshot tests look realistic
- Canned narratives should reference the actual zone labels and tier levels from input
- Error path: mock can also simulate LLM failures for error handling tests
- The trait boundary is where we'd swap in a different LLM provider (e.g., local model) later
