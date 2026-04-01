---
id: S-038
epic: E-015
title: Fix Remaining Scenario Failures
status: open
priority: critical
tickets: [T-038-01, T-038-02, T-038-03]
---

## Goal

Fix the 2 remaining scenario test failures so the dashboard reaches 12 passing with 0 failures. Both are test-level bugs, not product bugs.

## Acceptance Criteria

- S.3.3 and S.INFRA.2 pass with DATABASE_URL
- No regressions in the other 10 passing scenarios
- Docker Compose setup documented in a `just` recipe for easy DB-backed scenario runs
