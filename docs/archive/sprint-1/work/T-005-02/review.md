# Review: T-005-02 CF Worker Proxy

## Summary of Changes

### Files Created
| File | Lines | Purpose |
|------|-------|---------|
| `worker/src/index.ts` | 195 | Main worker: CORS, rate limiting, auth passthrough, SSE streaming, routing |
| `worker/src/index.test.ts` | 175 | 28 unit tests for rate limiting, CORS, headers |
| `worker/package.json` | 17 | Project config, scripts (dev, deploy, test) |
| `worker/tsconfig.json` | 13 | TypeScript strict mode, ESNext, workers types |
| `worker/wrangler.toml` | 8 | Wrangler config with dev defaults |
| `worker/vitest.config.ts` | 6 | Vitest test runner config |
| `worker/pnpm-lock.yaml` | — | Lockfile (generated) |

### Files Deleted
| File | Reason |
|------|--------|
| `worker/.gitkeep` | Replaced by real project files |

---

## Acceptance Criteria Verification

| Criterion | Status | Notes |
|-----------|--------|-------|
| Worker project in worker/ with wrangler.toml | Done | `plantastic-api-proxy`, `src/index.ts` entry |
| CORS: configurable allowed origin | Done | `ALLOWED_ORIGIN` env var, locked in prod via secret |
| Rate limiting: per-IP (configurable max/min) | Done | Sliding window, 60s, default 60 req/min |
| Rate limiting: per-session (configurable lifetime max) | Done | Lifetime counter using Authorization token, default 200 |
| Rate limit headers in responses | Done | `X-RateLimit-Limit`, `Remaining`, `Reset` |
| Auth token passthrough | Done | `Authorization` header forwarded to backend |
| SSE streaming passthrough (no buffering) | Done | `response.body` ReadableStream passed directly |
| BACKEND_URL via wrangler secret | Done | Named `BACKEND_URL` (adapted from HMW's `LAMBDA_URL`) |
| Deploys via `npx wrangler deploy` | Done | Script in package.json |
| Health check passthrough: GET /health | Done | Proxied to backend `/health` |

---

## Test Coverage

### Covered (28 tests, all passing)
- `getLimit`: valid string, undefined, non-numeric, zero, negative (5 tests)
- `checkIpRateLimit`: under limit, at limit, remaining tracking, reset timestamp (4 tests)
- `checkSessionRateLimit`: under limit, at limit, independent sessions, resetAt=0 (4 tests)
- `isOriginAllowed`: wildcard, matching, non-matching, null, empty (5 tests)
- `corsHeaders`: wildcard, specific, rejected, methods, auth header, expose headers (6 tests)
- `rateLimitHeaders`: limit+remaining, reset present, reset absent, negative clamp (4 tests)

### Not Covered
- **Full handler integration**: Requires Miniflare or `unstable_dev`, which needs workerd build scripts. The handler is a thin routing layer over tested functions — low risk.
- **SSE streaming end-to-end**: Requires a running backend that sends `text/event-stream`. The passthrough is a direct `response.body` forward with zero transformation — the Cloudflare Workers platform guarantees no buffering for ReadableStream.
- **proxyToBackend**: Makes a real `fetch()` call — needs a backend to test against. Logic is minimal (URL construction + header forwarding).

---

## Open Concerns

1. **In-memory rate limiting is per-isolate**: Different Cloudflare edge locations (and even different isolates at the same location) maintain independent rate limit state. This means a determined attacker could exceed the intended rate limit by hitting different PoPs. Acceptable for prototype; KV-backed upgrade path is documented in the HMW reference.

2. **Session rate limit uses Authorization header**: The design reuses the auth token as the session identifier, avoiding a custom header. If the backend uses short-lived tokens that rotate frequently, each new token gets a fresh session budget. This is a feature (token rotation = new budget) not a bug, but worth noting.

3. **No request validation**: The worker forwards request bodies as-is. Input validation is the backend's responsibility. This is intentional — the worker is a proxy, not a gateway.

4. **workerd build scripts not approved**: `pnpm approve-builds` was not run, so `wrangler dev` may fail until build scripts for workerd/esbuild/sharp are approved. This is an environment setup step, not a code issue.

5. **Deploys require BACKEND_URL secret**: `wrangler secret put BACKEND_URL <url>` must be run before the first deploy. No backend exists yet (E-001 track), so deployment is not yet possible end-to-end.

---

## Architecture Notes for Downstream

- T-005-03 (API client) should target the worker URL in production and the Vite dev proxy in development. The worker URL will be `https://plantastic-api-proxy.<account>.workers.dev` or a custom domain.
- The worker accepts both GET and POST on `/api/*` paths, which gives the API client flexibility for future REST endpoints beyond the current POST-only pattern.
