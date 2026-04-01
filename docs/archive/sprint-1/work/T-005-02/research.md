# Research: T-005-02 CF Worker Proxy

## Existing Codebase State

### worker/ directory
Empty — contains only `.gitkeep`. No existing worker code, config, or dependencies.

### web/ directory (T-005-01, done)
SvelteKit 5 scaffold is complete:
- Cloudflare Pages adapter configured (`svelte.config.js`)
- Vite dev proxy: `/api` → `process.env.API_URL || http://localhost:3000`
- pnpm package manager, TypeScript strict, Tailwind CSS 4
- Route skeleton in place (landing, dashboard, project workspace, catalog, settings, client view)
- No API client module yet (that's T-005-03, depends on this ticket)

### infra/ directory
Empty — `.gitkeep` only. No IaC yet.

### Broader context
- Plantastic is a Rust monorepo (Axum backend, PostGIS, S3). The CF Worker sits between the SvelteKit frontend and the Rust API, not a Lambda.
- No backend API exists yet (E-001 track). The worker must be buildable and deployable independently.
- T-005-03 (route skeleton + API client) depends on this ticket — the API client will target the worker URL in production and the Vite dev proxy in development.

---

## Reference Implementation: HMW Workshop

The proven CF Worker proxy lives at `/Volumes/ext1/swe/repos/how-might-we/worker/`. Single-file architecture: `src/index.ts` (322 lines).

### Architecture
- **Entry point:** `export default { fetch() }` handler using `ExportedHandler<Env>` pattern
- **All logic in one file** — no separate modules. Helper functions for CORS, rate limiting, proxy, Turnstile verification.
- **No runtime dependencies** — only dev deps: `wrangler`, `typescript`, `@cloudflare/workers-types`

### CORS
- `ALLOWED_ORIGIN` env var (default `*`, locked in production via wrangler secret)
- Preflight: `OPTIONS` → 204 with CORS headers, or 403 if origin not allowed
- `Access-Control-Allow-Methods: POST, OPTIONS`
- Exposes rate limit headers (`X-RateLimit-*`, `Retry-After`)
- `Access-Control-Max-Age: 86400` (24h preflight cache)

### Rate Limiting (Dual)
1. **Per-IP sliding window** (60s, default 20 req/min): in-memory `Map<IP, timestamp[]>` with 10k cleanup threshold
2. **Per-session lifetime counter** (default 50 req): in-memory `Map<SessionToken, count>` via `X-Session-Token` header
3. **Optional KV-backed mode**: same logic but uses Cloudflare KV for cross-isolate/cross-location consistency. Falls back to in-memory on KV errors.
4. **Rate limit headers**: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`, `Retry-After` (on 429)

### SSE Streaming Passthrough
- `lambdaResponse.body` (ReadableStream) passed directly to `new Response()` — zero buffering
- Lambda's `Content-Type: text/event-stream` header preserved
- Response status + all Lambda headers forwarded, then CORS + rate limit headers overlaid

### Auth/Security
- `X-Session-Token` used for session rate limiting (not forwarded to backend)
- `X-Turnstile-Token` for optional bot verification via Cloudflare Turnstile API
- Only `Content-Type` header forwarded to backend (safe header filtering)
- `LAMBDA_URL` is a wrangler secret (never exposed to client)

### Routing
- Only proxies `/api/*` paths — returns 404 for everything else
- Only allows POST — returns 405 for other methods
- Constructs target URL: `LAMBDA_URL + request.pathname`

---

## Key Differences: HMW → Plantastic

| Aspect | HMW Workshop | Plantastic |
|--------|-------------|------------|
| Backend | AWS Lambda (Go) | Rust/Axum (not yet built) |
| Auth | X-Session-Token (anonymous) | Authorization header passthrough (JWT/token) |
| Methods | POST only | GET + POST (health check, future REST endpoints) |
| Turnstile | Optional bot verification | Not needed initially |
| KV rate limiting | Optional upgrade | Not needed initially |
| Routing | /api/* only | /api/* + /health passthrough |

### Adaptations needed
1. **Authorization header passthrough**: HMW doesn't forward auth headers. Plantastic needs to forward `Authorization` to the Axum backend.
2. **GET support**: Health check (`GET /health`) and potentially future read endpoints need GET method support.
3. **No Turnstile**: Not in acceptance criteria. Can be added later.
4. **No KV rate limiting**: In-memory is sufficient for prototype. KV is a production upgrade path.
5. **Health check**: `GET /health` must pass through to backend and return its response.

---

## Cloudflare Workers Platform Constraints

- **Isolate model**: Each request may hit a different isolate. In-memory rate limiting is per-isolate, not global. Acceptable for prototype.
- **Execution time**: 30s default (free plan), 30s+ on paid. SSE streaming stays open until body completes.
- **Subrequest limits**: 50 subrequests per invocation (1 per proxy call = fine).
- **No `node:` modules**: Workers use Web APIs (`fetch`, `Request`, `Response`, `Headers`, `ReadableStream`).
- **Wrangler secrets**: `wrangler secret put KEY` for production values. `[vars]` in wrangler.toml for dev defaults.
- **Module format**: `export default { fetch() }` — the modern module worker syntax.

---

## Assumptions & Constraints

1. Worker deploys independently from frontend and backend — no shared build step.
2. `LAMBDA_URL` naming from HMW is misleading for Plantastic — we'll use `BACKEND_URL` (points to Axum).
3. The frontend dev server's Vite proxy (`/api → localhost:3000`) bypasses the worker entirely in development. The worker is only hit in staging/production.
4. T-005-03's API client will need to know the worker URL for production fetch calls.
5. No database, no filesystem — worker is purely stateless proxy + rate limiting.
