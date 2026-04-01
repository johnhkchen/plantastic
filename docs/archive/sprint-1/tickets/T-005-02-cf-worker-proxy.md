---
id: T-005-02
story: S-005
title: cf-worker-proxy
type: task
status: open
priority: high
phase: done
depends_on: [T-005-01]
---

## Context

The Cloudflare Worker sits between the frontend and Lambda API, handling edge concerns so the API stays focused on business logic. This is the proven pattern from HMW Workshop — CORS, rate limiting, auth passthrough, SSE streaming support.

## Acceptance Criteria

- Worker project in worker/ directory with wrangler.toml
- CORS handling: configurable allowed origin (locked to CF Pages URL in production)
- Rate limiting: per-IP (configurable max/minute) + per-session (configurable lifetime max)
- Rate limit headers in responses (X-RateLimit-Limit, Remaining, Reset)
- Auth token passthrough (Authorization header forwarded to Lambda)
- SSE streaming passthrough (no buffering — streams Lambda responses back to client)
- LAMBDA_URL configured via wrangler secret
- Deploys via `npx wrangler deploy`
- Health check passthrough: GET /health returns Lambda health response
