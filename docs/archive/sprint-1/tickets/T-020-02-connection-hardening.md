---
id: T-020-02
story: S-020
title: connection-hardening
type: task
status: open
priority: high
phase: done
depends_on: [T-020-01]
---

## Context

Neon's serverless Postgres can cold-start when Lambda also cold-starts, causing a documented hang in sqlx/tokio-postgres. The connection setup in pt-repo needs explicit timeouts, retry logic, and support for Neon's pooled connection strings (PgBouncer in transaction mode).

## Acceptance Criteria

### Connection config
- `create_pool()` in pt-repo accepts connection options:
  - `connect_timeout`: default 15 seconds
  - `max_connections`: configurable (default 5 for Lambda, higher for local)
  - `min_connections`: 0 (allow scale-to-zero)
  - `acquire_timeout`: default 10 seconds
- Pooled connection string support (Neon's `-pooler` hostname suffix)
- `sslnegotiation=direct` support in connection string for faster TLS handshake

### Retry logic
- Connection establishment retries on transient failures (3 attempts, exponential backoff starting at 500ms)
- Retry only on connection errors, not query errors
- Log each retry attempt at WARN level

### Validation
- Test with local Postgres (Docker Compose) — immediate connection, no retries
- Document expected Neon cold-start behavior and connection timing
- Measure: local connection time vs Neon pooled vs Neon direct

### Compatibility
- Same `create_pool()` works for local Docker Postgres, Neon pooled, and Neon direct
- Connection mode determined entirely by `DATABASE_URL` — no code changes between environments
