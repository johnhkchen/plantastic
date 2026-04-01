---
id: T-008-02
story: S-008
title: quote-scenario-two-star
type: task
status: open
priority: high
phase: done
depends_on: [T-008-01]
---

## Context

Upgrade S.3.1 and S.3.2 scenario tests from ★☆☆☆☆ to ★★. Instead of testing pt-quote with in-memory structs, the scenarios now exercise the full path: create a project via API, add zones, assign materials, fetch the quote via GET /quote/:tier, and verify the response.

This proves the computation is reachable, not just correct.

## Acceptance Criteria

- S.3.1 scenario: creates project + zones + materials + assignments via API, fetches quote, asserts same arithmetic as current test
- S.3.2 scenario: same flow for three tiers, asserts Good < Better < Best from API response
- Both scenarios return ScenarioOutcome::Pass(Integration::TwoStar)
- Old ★☆☆☆☆ tests preserved as unit-level regression tests (they still run, just not the scenario gate)
- Claim milestones: "Axum API: routes + Lambda deployment" and "PostGIS schema + sqlx repository layer" if not already claimed
