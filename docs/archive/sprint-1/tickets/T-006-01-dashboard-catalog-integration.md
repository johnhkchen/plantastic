---
id: T-006-01
story: S-006
title: dashboard-catalog-integration
type: task
status: open
priority: high
phase: done
depends_on: [T-004-02, T-005-03]
---

## Context

This is the convergence ticket where both tracks meet. The frontend's API client connects to the live Lambda API through the CF Worker proxy. The dashboard shows real projects from the database, and the catalog page manages real materials. This completes the first vertical slice through the entire stack: CF Pages → CF Worker → Lambda → PostGIS → response.

## Acceptance Criteria

- Dashboard page calls GET /projects and renders project cards (name, address, status, date)
- Create project flow: address input → POST /projects → redirect to new project page
- Project page calls GET /projects/:id and displays project details with zone count
- Catalog page calls GET /materials and renders material list (name, category, unit, price)
- Add material flow: form → POST /materials → material appears in list
- Edit material: inline edit → PATCH /materials/:id → updates in list
- Delete material: confirm → DELETE /materials/:id → removed from list
- Environment wiring: CF Worker LAMBDA_URL points to deployed Lambda Function URL
- Frontend PUBLIC_API_URL points to CF Worker URL
- Full round-trip verified: create project in UI → visible in database → visible in UI
- Error states: loading spinners, empty states, API error display
