---
id: T-023-01
story: S-023
title: baseline-polish-audit
type: task
status: open
priority: medium
phase: ready
depends_on: [T-022-01]
---

## Context

After T-022-01 adds the polish dimension, every scenario defaults to ★☆☆☆☆ polish. This ticket audits each passing scenario against the polish rubric and adjusts ratings if warranted.

## Acceptance Criteria

- Review each of the 8 passing scenarios against the polish rubric:
  - ★☆☆☆☆: No loading/error/empty states, raw styling
  - ★★☆☆☆: Has loading indicators, error messages, empty state prompts
- For scenarios that are pure computation (S.1.1, S.1.3, S.2.2): polish N/A or auto ★★★☆☆ (no UX to polish)
- For API-only scenarios (S.1.2, S.2.1): assess whether the API returns helpful errors
- For UI scenarios (S.2.4, S.3.1, S.3.2): assess actual frontend state
- Write review artifact in `docs/active/work/T-023-01/review.md` with per-scenario rationale
- Update any scenario polish ratings that differ from ★☆☆☆☆
- `just check` passes

## Decision: Pure Computation Scenarios

Scenarios at ★☆☆☆☆ integration (computation only, no API/UI) need a policy decision:
- **Option A**: Polish is N/A for computation-only scenarios → auto-rate at ★★★★★ (no UX to critique)
- **Option B**: Polish applies to computation API too (error messages, input validation) → rate honestly

Recommend **Option A** for ★☆☆☆☆ integration scenarios. Polish becomes meaningful at ★★☆☆☆+ where a user or developer interacts with an interface.
