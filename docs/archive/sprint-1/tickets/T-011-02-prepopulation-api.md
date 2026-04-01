---
id: T-011-02
story: S-011
title: prepopulation-api
type: task
status: open
priority: medium
phase: done
depends_on: [T-011-01, T-004-02]
---

## Context

Wire pt-satellite into the API so creating a project from an address triggers pre-population. The baseline data is stored on the project and visible when the project loads in the frontend.

## Acceptance Criteria

- POST /projects with address triggers pt-satellite baseline generation
- Baseline stored on project record (lot polygon, trees, sun grid reference)
- GET /projects/:id returns baseline data
- Frontend project page shows lot boundary and detected trees if baseline exists
- Async if needed — baseline generation can run as a background job with status polling
- Upgrade S.1.2 scenario to ★★ if API round-trip is verified
