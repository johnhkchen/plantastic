---
id: E-002
title: API & Frontend Skeleton
status: open
sprint: 1
---

# E-002: API & Frontend Skeleton

## Goal

Stand up the deployment infrastructure so domain crates have an HTTP surface and a UI to connect to. At the end of this epic, the Axum API serves project and material CRUD on Lambda, the SvelteKit frontend renders a dashboard with real data, and the CF Worker proxy handles edge concerns.

Track B (frontend + worker) runs in parallel with E-001's critical path, then converges at integration.

## Stories

- **S-004**: Axum API on Lambda — API skeleton, SST config, CRUD routes
- **S-005**: Frontend & Edge Layer — SvelteKit scaffold, CF Worker proxy, routing shell
- **S-006**: End-to-End Integration — connect frontend to API, dashboard shows real projects

## Success Criteria

- API deploys to Lambda and responds to health check
- Project and material CRUD routes work end-to-end (create, read, update, delete)
- SvelteKit app deploys to CF Pages with working routing
- CF Worker proxies requests to Lambda with CORS and rate limiting
- Dashboard page lists projects fetched from the live API
