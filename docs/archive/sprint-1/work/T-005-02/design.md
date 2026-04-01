# Design: T-005-02 CF Worker Proxy

## Options Evaluated

### Option A: Direct Port of HMW Worker
Copy the HMW `index.ts` nearly verbatim, changing only the env var names and adding Authorization passthrough.

**Pros:** Proven pattern, minimal risk, fast to implement.
**Cons:** Carries HMW-specific baggage (Turnstile, POST-only restriction, KV plumbing) that isn't in acceptance criteria. Dead code from day one.

### Option B: Simplified Single-File Worker (Adapted from HMW)
Take the HMW architecture (single `index.ts`, helper functions, `ExportedHandler<Env>`) but only implement what's in the acceptance criteria. Drop Turnstile, KV-backed rate limiting, and POST-only restriction.

**Pros:** Clean, matches AC exactly, no dead code, still follows the proven single-file pattern. Easy to add Turnstile/KV later if needed.
**Cons:** Slight re-implementation effort vs copy-paste. Marginal.

### Option C: Multi-File Modular Worker
Split into `cors.ts`, `rateLimit.ts`, `proxy.ts`, `types.ts`, etc. Separate concerns into modules.

**Pros:** Theoretically more maintainable at scale.
**Cons:** Over-engineered for ~200 lines of logic. HMW proved single-file works fine at this complexity. Adds import ceremony, harder to review as a unit.

---

## Decision: Option B — Simplified Single-File Worker

**Rationale:** The HMW pattern is proven and John has validated it in production. But carrying unused features (Turnstile, KV) violates the "don't add features beyond what was asked" principle. A focused implementation that matches the acceptance criteria exactly is the right call. The single-file architecture is appropriate for the complexity level (~250 lines).

---

## Design Decisions

### 1. Routing: /api/* + /health

HMW only proxies `/api/*`. Plantastic needs:
- `GET /health` — passthrough to backend health endpoint
- `POST /api/*` — standard API proxy
- `GET /api/*` — needed for future REST read endpoints

**Design:** Allow GET and POST on all paths that match `/api/*` or `/health`. Return 404 for everything else. Return 405 for non-GET/POST methods.

### 2. CORS: Configurable Origin

Same pattern as HMW:
- `ALLOWED_ORIGIN` env var, defaults to `*` for development
- Locked to CF Pages URL in production via `wrangler secret put`
- Preflight (OPTIONS) returns 204 with CORS headers
- Allow methods: `GET, POST, OPTIONS`
- Allow headers: `Content-Type, Authorization` (no Turnstile/Session headers)
- Expose headers: `X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset, Retry-After`

### 3. Rate Limiting: In-Memory Only

No KV for now — acceptance criteria says "per-IP" and "per-session" with configurable limits, not distributed.

**Per-IP (sliding window, 60s):**
- `RATE_LIMIT_IP_MAX` env var (default: 60 — higher than HMW's 20 since we're supporting GET reads too)
- In-memory `Map<IP, timestamp[]>` with sliding window
- IP from `cf-connecting-ip` header (Cloudflare provides this)
- 10k cleanup threshold

**Per-session (lifetime counter):**
- `RATE_LIMIT_SESSION_MAX` env var (default: 200 — higher than HMW's 50 for a full-session app)
- Session token from `Authorization` header (reuse the auth token as session identifier)
- In-memory `Map<token, count>`
- 10k cleanup threshold

**Rate limit headers on every response:**
- `X-RateLimit-Limit` — the per-IP max
- `X-RateLimit-Remaining` — requests left in window
- `X-RateLimit-Reset` — Unix timestamp (seconds) when window resets

### 4. Auth Token Passthrough

HMW doesn't forward auth headers. Plantastic forwards `Authorization`:
- Read `Authorization` header from client request
- Forward it to backend in the proxied request
- No validation at the worker level — the Axum backend validates tokens
- If no `Authorization` header, still proxy the request (backend decides whether auth is required)

### 5. SSE Streaming Passthrough

Identical to HMW — the key insight is zero buffering:
- `fetch()` to backend returns a `Response` with `ReadableStream` body
- Pass `response.body` directly to `new Response()` — no `await response.text()` or buffering
- Backend's `Content-Type: text/event-stream` header is preserved
- CORS and rate limit headers are overlaid on the response

### 6. Health Check

`GET /health`:
- Proxied to backend's `/health` endpoint
- Returns whatever the backend returns (status code, body)
- CORS headers applied (so frontend can health-check from browser)
- Rate limiting still applies (prevents health-check abuse)

### 7. Environment Variables

| Variable | Source | Default | Description |
|----------|--------|---------|-------------|
| `BACKEND_URL` | wrangler secret | (none, required) | Axum backend URL |
| `ALLOWED_ORIGIN` | wrangler.toml / secret | `*` | CORS allowed origin |
| `RATE_LIMIT_IP_MAX` | wrangler.toml | `60` | Max requests per IP per minute |
| `RATE_LIMIT_SESSION_MAX` | wrangler.toml | `200` | Max requests per session lifetime |

### 8. Error Responses

All errors return JSON with CORS headers:
```json
{ "error": "Rate limit exceeded" }
```
Status codes: 404 (not found), 405 (method not allowed), 429 (rate limited), 502 (backend unavailable/not configured).

---

## What Was Rejected

1. **Turnstile bot verification** — not in AC, adds complexity and external dependency. Easy to add later as a middleware-style check.
2. **KV-backed rate limiting** — not in AC, adds cost and complexity. In-memory is fine for prototype scale. Upgrade path is clear from HMW.
3. **Multi-file architecture** — over-engineered for ~250 lines. The single-file pattern from HMW is proven.
4. **Custom session token header** — HMW uses `X-Session-Token`. Plantastic can reuse the `Authorization` header as a session identifier, avoiding an extra custom header. The auth token is unique per session by nature.
5. **WebSocket support** — not in AC. SSE streaming is the required pattern.
6. **Request body transformation** — no need to modify request/response bodies. Pure passthrough.
