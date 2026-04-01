# Proposal Fixtures

These JSON files contain real LLM output captured from `ClaudeCliGenerator`.
They are used by `MockProposalGenerator` to return realistic, deterministic
proposal narratives in tests — without calling an LLM.

## How it works

1. `sample_input.json` — the canonical test input (company, project, tiers, zones)
2. `sample_output.json` — real Claude output for that input, parsed by BAML

`MockProposalGenerator` loads `sample_output.json` at compile time via
`include_str!`. Tests get production-quality narrative text for free.

## How to regenerate

When the ProposalContent schema changes, or you want fresher copy:

```bash
cargo run -p pt-proposal --example regenerate_fixture
```

This calls `ClaudeCliGenerator` (your subscription, zero API cost), writes
the result to `sample_output.json`, and prints a diff summary. Review the
output, then commit the updated fixture.

## How to add a new fixture

1. Create a new input JSON (e.g., `commercial_input.json`)
2. Add it to the `regenerate_fixture` example's input list
3. Run the regeneration command
4. Use it in tests: `MockProposalGenerator::from_fixture("commercial")`
