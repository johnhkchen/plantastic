# Structure: T-005-02 CF Worker Proxy

## Files Created

### worker/src/index.ts (~250 lines)
Main worker entry point. Single file containing all logic.

**Sections (top to bottom):**

1. **`Env` interface** — typed environment bindings (`BACKEND_URL`, `ALLOWED_ORIGIN`, `RATE_LIMIT_IP_MAX`, `RATE_LIMIT_SESSION_MAX`)

2. **`RateLimitResult` interface** — `{ allowed, remaining, resetAt }`

3. **Rate limiting helpers:**
   - `getLimit(val, fallback)` — parse env var string to number with fallback
   - `ipTimestamps: Map<string, number[]>` — module-level sliding window state
   - `checkIpRateLimit(ip, max): RateLimitResult` — per-IP sliding window (60s)
   - `sessionCounts: Map<string, number>` — module-level session counter state
   - `checkSessionRateLimit(token, max): RateLimitResult` — per-session lifetime counter

4. **Header helpers:**
   - `rateLimitHeaders(result, max): Record<string, string>` — build `X-RateLimit-*` headers
   - `isOriginAllowed(env, origin): boolean` — check origin against `ALLOWED_ORIGIN`
   - `corsHeaders(env, origin): Record<string, string>` — build CORS response headers
   - `jsonError(message, status, env, origin, extra?): Response` — error response factory

5. **Proxy function:**
   - `proxyToBackend(request, env, origin, extraHeaders): Promise<Response>` — construct target URL, forward safe headers (Content-Type, Authorization), pass through response body as stream

6. **Main handler:**
   - `export default { fetch(request, env) }` — routing, CORS preflight, rate limit checks, proxy dispatch

### worker/wrangler.toml
Wrangler configuration:
```toml
name = "plantastic-api-proxy"
main = "src/index.ts"
compatibility_date = "2025-03-01"

[vars]
ALLOWED_ORIGIN = "*"
RATE_LIMIT_IP_MAX = "60"
RATE_LIMIT_SESSION_MAX = "200"
```
No KV bindings. `BACKEND_URL` set via `wrangler secret put`.

### worker/package.json
```json
{
  "name": "plantastic-api-proxy",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "wrangler dev",
    "deploy": "wrangler deploy"
  },
  "devDependencies": {
    "@cloudflare/workers-types": "^4.20250312.0",
    "typescript": "^5.9.0",
    "wrangler": "^4.0.0",
    "vitest": "^3.1.0"
  }
}
```
Vitest added for unit testing the rate limiting and CORS logic.

### worker/tsconfig.json
```json
{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "lib": ["ESNext"],
    "types": ["@cloudflare/workers-types"],
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true
  },
  "include": ["src/**/*.ts"]
}
```

### worker/src/index.test.ts (~150 lines)
Unit tests for rate limiting and CORS logic. Tests the pure functions directly (no need to spin up a worker):
- `checkIpRateLimit`: allows up to max, blocks after, sliding window resets
- `checkSessionRateLimit`: allows up to max, blocks after
- `isOriginAllowed`: wildcard, matching, non-matching
- `corsHeaders`: correct headers for wildcard vs specific origin
- `rateLimitHeaders`: correct X-RateLimit-* values

Integration-style tests for the full handler using `unstable_dev` or Miniflare are deferred — the proxy needs a real backend to test SSE streaming, and no backend exists yet.

---

## Files Modified

### worker/.gitkeep
**Deleted** — no longer needed once real files exist.

---

## Files NOT Modified

- `web/` — no changes. The Vite dev proxy already handles dev-time API routing.
- `docs/active/tickets/T-005-02-cf-worker-proxy.md` — phase/status managed by Lisa.
- `Cargo.toml`, `Cargo.lock` — worker is a standalone JS/TS project, no Rust dependency.

---

## Module Boundaries

### Public Interface (to consumers)
The worker exposes an HTTP interface, not a code API:
- `GET /health` → proxied to `BACKEND_URL/health`
- `GET /api/*` → proxied to `BACKEND_URL/api/*`
- `POST /api/*` → proxied to `BACKEND_URL/api/*`
- `OPTIONS *` → CORS preflight response
- Everything else → 404

### Internal Organization
All logic is internal to `src/index.ts`. No exports consumed by other packages. The test file imports specific functions for unit testing — these functions are exported but only used by tests.

### Dependency Direction
```
SvelteKit (web/) → CF Worker (worker/) → Axum Backend (not yet built)
                    ↑ T-005-03 API client targets this in production
```

---

## Ordering of Changes

1. Remove `worker/.gitkeep`
2. Create `worker/package.json`, `worker/tsconfig.json`, `worker/wrangler.toml`
3. Create `worker/src/index.ts` (main implementation)
4. Install dependencies (`pnpm install` in worker/)
5. Create `worker/src/index.test.ts`
6. Run tests to verify
7. Verify `wrangler dev` starts without errors (with mock backend or graceful failure)
