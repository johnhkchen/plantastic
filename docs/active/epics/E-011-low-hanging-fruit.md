---
id: E-011
title: Low-Hanging Fruit Scenarios
status: open
priority: high
sprint: 2
---

## Context

Sprint 1 ended with 8 passing and 9 NotImplemented scenarios. Three of the nine have most or all engineering prereqs already delivered — they just need scenario tests written and (in some cases) thin milestones claimed.

Flipping these costs minimal engineering effort but meaningfully advances the dashboard: +2 infrastructure gates (proving the deployed stack works) and +1 crew handoff scenario.

## Targets

| Scenario | Prereqs Met | Remaining Work |
|----------|-------------|----------------|
| S.4.3 Material callouts | 2/2 | Write test using pt-materials + materials API |
| S.INFRA.2 Tenant isolation | 3/4 | Write test with existing api_helpers; claim pt-tenant |
| S.INFRA.1 Full-stack round-trip | 5/8 | Write test with existing api_helpers; claim SvelteKit + pt-project milestones |

## Stories

- S-024: Crew Handoff — Material Callouts (S.4.3)
- S-025: Infrastructure Validation (S.INFRA.1, S.INFRA.2)

## Success Criteria

- S.4.3 passes at ★☆☆☆☆ or ★★☆☆☆
- S.INFRA.1 passes at ★★☆☆☆ (API round-trip, no full deployed stack yet)
- S.INFRA.2 passes at ★★☆☆☆ (API tenant isolation verified)
- Unclaimed milestones (SvelteKit frontend, pt-tenant, pt-project) claimed with notes
- `just check` passes, no regressions
