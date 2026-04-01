---
id: S-025
epic: E-011
title: Infrastructure Validation
status: open
priority: high
tickets: [T-025-01, T-025-02, T-025-03]
---

## Goal

Flip S.INFRA.1 (full-stack round-trip) and S.INFRA.2 (tenant isolation) from NotImplemented to Pass. Both scenarios have most prereqs delivered and existing test scaffolding (api_helpers from S.3.1/S.3.2) that can be reused.

Also claim the milestones that are effectively delivered but unclaimed: SvelteKit frontend + CF Worker proxy, pt-tenant (TenantRepo exists in pt-repo), pt-project (domain model exists).

## Acceptance Criteria

- S.INFRA.2 passes at ★★☆☆☆ (tenant isolation via API)
- S.INFRA.1 passes at ★★☆☆☆ (full CRUD + quote round-trip via API)
- Milestones claimed: SvelteKit frontend, pt-tenant, pt-project, pt-quote
- `just check` passes
