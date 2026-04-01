---
id: S-006
epic: E-002
title: End-to-End Integration
status: open
priority: high
dependencies:
  - S-004
  - S-005
---

# S-006: End-to-End Integration

## Purpose

Connect the frontend to the live API through the worker proxy. This is the convergence point where both tracks meet. At the end, a landscaper can create a project, see it on the dashboard, and manage their material catalog — the first vertical slice through the entire stack.

## Scope

- Wire API client to live Lambda endpoints via CF Worker
- Dashboard page: list projects from API, create new project (address input)
- Material catalog page: list, add, edit materials from API
- Deploy verification: full round-trip from CF Pages → CF Worker → Lambda → PostGIS and back
- Environment configuration: Doppler secrets, SST outputs piped to worker/frontend

## Tickets

- T-006-01: Dashboard + catalog wired to live API
