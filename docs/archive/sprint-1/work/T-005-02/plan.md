# Plan: T-005-02 CF Worker Proxy

## Step 1: Scaffold worker project

**Actions:**
- Delete `worker/.gitkeep`
- Create `worker/package.json` with dev dependencies (wrangler, @cloudflare/workers-types, typescript, vitest)
- Create `worker/tsconfig.json` with strict mode, ESNext target, workers types
- Create `worker/wrangler.toml` with dev defaults

**Verification:** Files exist with correct content. `cat` each file to confirm.

---

## Step 2: Implement main worker handler

**Actions:**
- Create `worker/src/index.ts` with complete implementation:
  - `Env` and `RateLimitResult` interfaces
  - Per-IP sliding window rate limiter (in-memory Map)
  - Per-session lifetime counter (in-memory Map)
  - CORS helpers (origin validation, header generation, preflight)
  - JSON error response factory
  - Proxy function (forwards Content-Type + Authorization, streams response body)
  - Main fetch handler (routing, CORS, rate limiting, proxy dispatch)

**Verification:** TypeScript compiles without errors (`npx tsc --noEmit`).

---

## Step 3: Install dependencies

**Actions:**
- Run `cd worker && pnpm install`

**Verification:** `node_modules` exists, `pnpm-lock.yaml` generated.

---

## Step 4: Verify TypeScript compilation

**Actions:**
- Run `cd worker && npx tsc --noEmit`

**Verification:** No type errors.

---

## Step 5: Write unit tests

**Actions:**
- Create `worker/vitest.config.ts` (minimal config)
- Create `worker/src/index.test.ts` with tests:
  - IP rate limiter: allows requests under limit, blocks at limit, sliding window behavior
  - Session rate limiter: allows under limit, blocks at limit, independent sessions
  - `isOriginAllowed`: wildcard allows all, specific origin matches, rejects mismatch
  - `corsHeaders`: wildcard vs specific, empty on rejected origin
  - `rateLimitHeaders`: correct header values
  - Integration: full handler via Miniflare/unstable_dev (if feasible without backend)

**Verification:** `pnpm vitest run` passes all tests.

---

## Step 6: Verify wrangler dev starts

**Actions:**
- Run `cd worker && npx wrangler dev` briefly to confirm the worker starts
- Test against it with curl if a mock backend is available, otherwise confirm startup without errors

**Verification:** Worker starts, responds to `GET /` with 404 JSON error, `OPTIONS /api/test` with 204 preflight.

---

## Testing Strategy

### Unit Tests (Step 5)
- **Rate limiting functions**: Pure functions, easy to test in isolation. Test boundary conditions (exactly at limit, one over, sliding window reset).
- **CORS functions**: Pure functions. Test wildcard, matching origin, non-matching origin.
- **Header builders**: Pure functions. Verify correct header names and values.

### Manual Smoke Tests (Step 6)
- `curl -X OPTIONS http://localhost:8787/api/test` → 204 with CORS headers
- `curl http://localhost:8787/health` → 502 (no backend configured) or proxy attempt
- `curl http://localhost:8787/api/test` → 502 (no backend) with rate limit headers
- `curl http://localhost:8787/random` → 404

### Integration Tests (Deferred)
- Full SSE streaming passthrough requires a running backend
- End-to-end flow (frontend → worker → backend) tested when API exists
- Can be tested with a simple mock SSE server when T-005-03 needs it

---

## Commit Plan

1. **Commit after Step 2**: "Add CF Worker proxy with CORS, rate limiting, auth passthrough, and SSE streaming"
2. **Commit after Step 5**: "Add unit tests for CF Worker rate limiting and CORS"

Two atomic commits: implementation, then tests.
