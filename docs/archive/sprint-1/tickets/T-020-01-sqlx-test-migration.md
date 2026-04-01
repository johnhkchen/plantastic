---
id: T-020-01
story: S-020
title: sqlx-test-migration
type: task
status: open
priority: high
phase: done
depends_on: [T-019-01]
---

## Context

pt-repo integration tests currently use a manual `setup_test_db()` helper and are `#[ignore]`'d because they require Postgres. Migrate to sqlx's built-in `#[sqlx::test]` attribute, which creates an ephemeral database per test with migrations auto-applied, and un-ignore all tests.

## Acceptance Criteria

### sqlx::test migration
- All tests in `crates/pt-repo/tests/` use `#[sqlx::test(migrations = "../../migrations")]`
- Each test receives a `PgPool` parameter — no manual pool creation
- Remove `common/mod.rs` setup/teardown helpers (or reduce to seed data only)
- Remove all `#[ignore]` annotations from Postgres-dependent tests
- Tests are fully isolated — no shared state, no cleanup needed

### Test gating
- `just test` runs only unit tests (no database dependency): `cargo test --workspace` with feature gating or test filtering
- `just test-integration` runs repo integration tests: requires `DATABASE_URL` set
- Integration tests skip gracefully (not fail) when no database is available
- `just check` documents that full validation requires `just dev-db` running

### Verification
- `just dev-db && just test-integration` passes all previously-ignored tests
- Each test creates and destroys its own database — run tests in parallel without interference
- Test suite completes in < 30 seconds (measure and report)
