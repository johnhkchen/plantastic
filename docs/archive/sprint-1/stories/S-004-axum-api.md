---
id: S-004
epic: E-002
title: Axum API on Lambda
status: open
priority: high
dependencies:
  - S-003
---

# S-004: Axum API on Lambda

## Purpose

Give the domain crates an HTTP surface. The Axum API is the single backend entry point — it handles REST routes for project and material CRUD, wires the repository layer, and deploys to Lambda via SST with scale-to-zero.

## Scope

- Axum router with Lambda runtime detection (same binary runs locally and on Lambda)
- SST configuration: Lambda function (provided.al2023, arm64), Function URL with RESPONSE_STREAM mode
- Health endpoint for deploy verification
- Project CRUD routes: create, get, list, delete
- Zone CRUD routes: list, bulk update, add, patch, delete (scoped to project)
- Material CRUD routes: list, create, patch, delete (scoped to tenant)
- Tier assignment routes: get tiers, set assignments per tier
- Error handling, request validation, JSON responses

## Tickets

- T-004-01: Axum skeleton + Lambda + SST
- T-004-02: Project, zone, and material CRUD routes
