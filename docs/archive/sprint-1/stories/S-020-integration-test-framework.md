---
id: S-020
epic: E-009
title: Integration Test Framework
status: open
priority: high
dependencies:
  - S-019
---

# S-020: Integration Test Framework

## Purpose

Integration tests that hit real Postgres are currently `#[ignore]`'d and run manually. This story makes them first-class: `#[sqlx::test]` provides per-test ephemeral databases, all ignored tests are un-ignored and wired to detect a running Postgres, and the connection layer handles Neon's cold-start behavior gracefully.

## Scope

- Migrate pt-repo tests from manual `setup_test_db()` to `#[sqlx::test]`
- Un-ignore all Postgres-dependent tests
- `just test-integration` recipe that requires a running database
- `just test` continues to run unit tests only (no database required)
- `just check` updated to include integration tests when database is available
- sqlx connection config with explicit timeouts, retry, pooled connection support
- Document the two-mode testing: `just test` (fast, no deps) vs `just test-integration` (needs Compose)

## Tickets

- T-020-01: sqlx::test migration + un-ignore tests
- T-020-02: Connection hardening (timeouts, retry, pooler support)
