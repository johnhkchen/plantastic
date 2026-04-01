# T-021-03 Design: Lambda → Neon Connection Validation

## Decision summary

1. Create a dedicated `scripts/validate-lambda-neon.sh` script for connection timing
2. Add a `/health/ready` endpoint that pings the database (readiness probe)
3. Update E-008 and T-017-02 documentation in-place
4. Record results in a `results.md` work artifact

---

## Option analysis

### Validation approach

**Option A: Instrument the Rust binary with timing endpoints**

Add `/debug/timing` route that returns connection pool metrics, retry counts, etc.
Pros: precise, programmatic. Cons: adds debug surface to production binary,
couples validation to application code, needs cleanup after.

**Option B: External bash script using `aws lambda invoke` + CloudWatch logs**

Script invokes Lambda (via Function URL or `aws lambda invoke`), measures wall-clock
response times, and optionally parses CloudWatch for internal timing.
Pros: zero code changes for timing, uses existing infrastructure. Cons: less precise
breakdown (can't separate Neon wake from TLS from query).

**Option C: Hybrid — readiness endpoint + external script**

Add a lightweight `/health/ready` that does `SELECT 1`, keep `/health` as-is for
liveness. External script calls both endpoints and measures timing. CloudWatch logs
from pool.rs already emit `elapsed_ms` for internal breakdown.

**Decision: Option C (hybrid)**

Rationale:
- `/health/ready` is operationally useful beyond this ticket (ALB health checks,
  deployment gates, monitoring). Not throwaway debug code.
- External script avoids coupling validation logic to the application.
- pool.rs already logs everything we need internally — CloudWatch is the source
  of truth for the internal breakdown.
- The script documents the procedure so it can be repeated.

### Readiness endpoint design

```
GET /health/ready → 200 {"status":"ok","db":"ok","latency_ms":N}
                  → 503 {"status":"degraded","db":"error","error":"..."}
```

Implementation: `SELECT 1` against the pool. Timeout at 5s (matches AC).
If pool acquire + query succeeds: 200. If it fails or times out: 503.
This directly addresses the AC "health endpoint returns 200 even on cold start
within acceptable time (< 5 seconds total)."

Keep existing `/health` as liveness (no DB dependency). Container orchestrators
and load balancers distinguish liveness from readiness — we should too.

### Validation script design

`scripts/validate-lambda-neon.sh <api-url>` performs:

1. **Cold start timing** — First request to `/health/ready` after deploy or long idle.
   Measures total response time (Lambda init + Neon wake + pool connect + query).

2. **Warm request timing** — Immediate second request. Measures steady-state latency.

3. **Idle recovery** — Script sleeps are impractical for 10 minutes in a CI-like
   context. Instead, document the manual procedure and provide a `--idle` flag
   that pauses and prompts the operator after 10 minutes.

4. **Concurrent cold starts** — Use `xargs -P10` or background `curl` processes
   to hit the endpoint simultaneously. Lambda auto-scales, creating 10 instances.
   Collect all response times.

5. **Retry verification** — Parse CloudWatch logs for retry WARN messages.
   Use `aws logs filter-log-events` with pattern filter.

Output: structured text with pass/fail per check, timing data.
Results saved to `docs/active/work/T-021-03/results.md` by the operator.

### Documentation updates

**E-008 epic changes:**
- Infrastructure table: Railway → Neon (with connection string details)
- Description: remove Railway-specific language, add Neon specifics
- "What's needed" section: Railway PostGIS item → completed via Neon

**T-017-02 ticket changes:**
- Title stays (kebab-case in filename, not worth renaming file)
- Railway PostGIS section → Neon PostGIS section with updated criteria
- Verification section → reference Neon connection strings and pooled endpoint

---

## Rejected alternatives

**Full integration test in Rust**: Considered adding a Rust integration test that
connects to Neon and measures timing. Rejected because: (a) the test would require
a live Neon connection string in CI, which is T-021-02's scope, not ours; (b) the
pool.rs unit tests already verify retry logic; (c) a bash script is more appropriate
for one-time validation that produces a human-readable report.

**Modifying pool.rs to emit metrics**: Considered adding `tracing::info!` spans for
each sub-phase (DNS, TCP, TLS, auth). Rejected because sqlx doesn't expose these
breakdowns, and adding TCP-level instrumentation is over-engineering for a validation
ticket. CloudWatch `elapsed_ms` is sufficient.

**Adding a `/debug/connections` endpoint**: Rejected. Would expose pool internals
in production. Not needed — CloudWatch logs have the data.

---

## Scenarios referenced

- **S.INFRA.1**: Infrastructure round-trip (deploy → health → CRUD → verify).
  The readiness endpoint directly supports this scenario.

## Dependencies confirmed

- T-021-01 (Neon provisioning): phase=done ✓
- T-017-01 (Lambda skeleton): phase=done (implied by existing deployment) ✓
- T-020-02 (connection hardening): phase=done ✓
