---
id: T-025-03
story: S-025
title: claim-unclaimed-milestones
type: task
status: open
priority: medium
phase: done
depends_on: [T-025-02]
---

## Context

Several milestones in `progress.rs` are unclaimed despite the underlying work being delivered in Sprint 1. This makes the dashboard underreport engineering progress.

## Acceptance Criteria

- Claim the following milestones with accurate notes:

  **SvelteKit frontend + CF Worker proxy** (unlocks S.INFRA.1, S.3.4)
  - Delivered by: T-005-02 (CF Worker) + T-005-03 (route skeleton + API client)
  - Note: SvelteKit app at web/, CF Worker proxy at worker/, route skeleton with API client layer, mock API for local dev

  **pt-project: Project/Zone/Tier model + GeoJSON serde** (unlocks S.3.1, S.3.2, S.3.4, S.INFRA.1)
  - Delivered by: T-002-01 (domain crates) + T-003-02 (repo layer)
  - Note: Project/Zone/Material/TierAssignment types in pt-repo with GeoJSON serde via ST_GeomFromGeoJSON/ST_AsGeoJSON

  **pt-quote: quantity takeoff engine** (unlocks S.3.1, S.3.2)
  - Delivered by: T-002-02 (pt-quote crate)
  - Note: compute_quote() takes zones + assignments + materials, returns Quote with line items. Hardscape area, softscape volume, edging perimeter, fill volume computations

  **pt-tenant: multi-tenant model + auth context** (unlocks S.INFRA.2)
  - Delivered by: T-003-02 (TenantRepo) + T-004-02 (X-Tenant-Id extractor)
  - Note: TenantRepo in pt-repo, X-Tenant-Id header extractor enforces tenant scoping on every API route

- Milestone count goes from 15/24 to 19/24
- `just check` passes
