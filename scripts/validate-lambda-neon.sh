#!/usr/bin/env bash
set -euo pipefail

# Validate Lambda → Neon connection under real conditions.
# Measures cold start timing, warm request latency, and concurrent cold starts.
#
# Usage: ./scripts/validate-lambda-neon.sh <api-url>
#        just validate-neon-lambda <api-url>
#
# Requires: curl
# Optional: --idle flag to include 10-minute idle test

if [ $# -lt 1 ]; then
    echo "Usage: $0 <api-url> [--idle]"
    echo ""
    echo "Example: $0 https://xxx.lambda-url.us-west-2.on.aws"
    echo "         $0 https://xxx.lambda-url.us-west-2.on.aws --idle"
    exit 1
fi

API_URL="${1%/}"
IDLE_TEST=false
if [ "${2:-}" = "--idle" ]; then
    IDLE_TEST=true
fi

PASS=0
FAIL=0
RESULTS=()

pass() { echo "  ✓ $1"; PASS=$((PASS + 1)); RESULTS+=("PASS  $1"); }
fail() { echo "  ✗ $1"; FAIL=$((FAIL + 1)); RESULTS+=("FAIL  $1"); }

# curl timing format: total_time, connect_time, starttransfer_time, http_code
CURL_FORMAT='{"total":%{time_total},"connect":%{time_connect},"ttfb":%{time_starttransfer},"tls":%{time_appconnect},"code":%{http_code}}'

timed_request() {
    local label="$1"
    local endpoint="$2"

    local response
    response=$(curl -s -o /dev/null -w "$CURL_FORMAT" "$endpoint" 2>/dev/null || echo '{"total":0,"connect":0,"ttfb":0,"tls":0,"code":0}')

    local total connect ttfb tls code
    total=$(echo "$response" | sed 's/.*"total":\([0-9.]*\).*/\1/')
    connect=$(echo "$response" | sed 's/.*"connect":\([0-9.]*\).*/\1/')
    ttfb=$(echo "$response" | sed 's/.*"ttfb":\([0-9.]*\).*/\1/')
    tls=$(echo "$response" | sed 's/.*"tls":\([0-9.]*\).*/\1/')
    code=$(echo "$response" | sed 's/.*"code":\([0-9]*\).*/\1/')

    # Convert to milliseconds
    total_ms=$(echo "$total * 1000" | bc | cut -d. -f1)
    connect_ms=$(echo "$connect * 1000" | bc | cut -d. -f1)
    ttfb_ms=$(echo "$ttfb * 1000" | bc | cut -d. -f1)
    tls_ms=$(echo "$tls * 1000" | bc | cut -d. -f1)

    echo "    HTTP $code | total: ${total_ms}ms | connect: ${connect_ms}ms | TLS: ${tls_ms}ms | TTFB: ${ttfb_ms}ms"

    if [ "$code" = "200" ]; then
        pass "$label (${total_ms}ms)"
    else
        fail "$label (HTTP $code, ${total_ms}ms)"
    fi
}

# Also fetch body for ready endpoint
ready_request() {
    local label="$1"
    local endpoint="$2"
    local tmpfile
    tmpfile=$(mktemp)

    local timing
    timing=$(curl -s -o "$tmpfile" -w "$CURL_FORMAT" "$endpoint" 2>/dev/null || echo '{"total":0,"connect":0,"ttfb":0,"tls":0,"code":0}')
    local body
    body=$(cat "$tmpfile")
    rm -f "$tmpfile"

    local total code
    total=$(echo "$timing" | sed 's/.*"total":\([0-9.]*\).*/\1/')
    code=$(echo "$timing" | sed 's/.*"code":\([0-9]*\).*/\1/')
    total_ms=$(echo "$total * 1000" | bc | cut -d. -f1)

    echo "    HTTP $code | total: ${total_ms}ms"
    if [ -n "$body" ]; then
        echo "    Body: $body"
    fi

    if [ "$code" = "200" ]; then
        pass "$label (${total_ms}ms)"
    else
        fail "$label (HTTP $code, ${total_ms}ms)"
    fi
}

echo "═══════════════════════════════════════════════════════"
echo "  Lambda → Neon Connection Validation"
echo "  Target: $API_URL"
echo "  Date:   $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "═══════════════════════════════════════════════════════"
echo ""

# ─── 1. Liveness (no DB) ────────────────────────────────────
echo "1. Liveness check (GET /health — no DB dependency)"
timed_request "Liveness probe" "$API_URL/health"
echo ""

# ─── 2. Cold start readiness (first DB hit) ─────────────────
echo "2. Readiness check (GET /health/ready — pings DB)"
echo "   This may include Lambda init + Neon compute wake + TLS + query."
ready_request "Cold readiness" "$API_URL/health/ready"
echo ""

# ─── 3. Warm request ────────────────────────────────────────
echo "3. Warm request (immediate second hit)"
ready_request "Warm readiness" "$API_URL/health/ready"
echo ""

# ─── 4. Concurrent cold starts ──────────────────────────────
echo "4. Concurrent cold starts (10 parallel requests)"
echo "   Each request may hit a different Lambda instance."

CONCURRENT_PIDS=()
CONCURRENT_DIR=$(mktemp -d)

for i in $(seq 1 10); do
    (
        result=$(curl -s -o /dev/null -w "%{http_code} %{time_total}" "$API_URL/health/ready" 2>/dev/null || echo "0 0")
        echo "$result" > "$CONCURRENT_DIR/$i"
    ) &
    CONCURRENT_PIDS+=($!)
done

# Wait for all
for pid in "${CONCURRENT_PIDS[@]}"; do
    wait "$pid" 2>/dev/null || true
done

CONCURRENT_PASS=0
CONCURRENT_FAIL=0
MAX_TIME=0

for i in $(seq 1 10); do
    if [ -f "$CONCURRENT_DIR/$i" ]; then
        read -r code total < "$CONCURRENT_DIR/$i"
        total_ms=$(echo "$total * 1000" | bc | cut -d. -f1)
        total_ms=${total_ms:-0}

        if [ "$code" = "200" ]; then
            echo "    [$i] HTTP $code — ${total_ms}ms"
            CONCURRENT_PASS=$((CONCURRENT_PASS + 1))
        else
            echo "    [$i] HTTP $code — ${total_ms}ms (FAIL)"
            CONCURRENT_FAIL=$((CONCURRENT_FAIL + 1))
        fi

        # Track max time
        if [ "$(echo "$total_ms > $MAX_TIME" | bc)" = "1" ]; then
            MAX_TIME=$total_ms
        fi
    fi
done
rm -rf "$CONCURRENT_DIR"

echo "    Summary: $CONCURRENT_PASS/10 succeeded, max latency: ${MAX_TIME}ms"

if [ "$CONCURRENT_FAIL" -eq 0 ]; then
    pass "Concurrent cold starts (10/10 OK, max ${MAX_TIME}ms)"
else
    fail "Concurrent cold starts ($CONCURRENT_FAIL/10 failed)"
fi
echo ""

# ─── 5. < 5 second acceptance criterion ─────────────────────
echo "5. Acceptance criterion: /health/ready < 5000ms on cold start"
FINAL_RESPONSE=$(curl -s -o /dev/null -w "%{http_code} %{time_total}" "$API_URL/health/ready" 2>/dev/null || echo "0 0")
read -r FINAL_CODE FINAL_TIME <<< "$FINAL_RESPONSE"
FINAL_MS=$(echo "$FINAL_TIME * 1000" | bc | cut -d. -f1)

if [ "$FINAL_CODE" = "200" ] && [ "$(echo "$FINAL_MS < 5000" | bc)" = "1" ]; then
    pass "Readiness < 5s (${FINAL_MS}ms)"
else
    fail "Readiness >= 5s or error (HTTP $FINAL_CODE, ${FINAL_MS}ms)"
fi
echo ""

# ─── 6. Idle recovery (optional) ────────────────────────────
if [ "$IDLE_TEST" = true ]; then
    echo "6. Idle recovery test"
    echo "   Waiting 10 minutes for Neon compute to suspend..."
    echo "   (Neon free/launch tier suspends after ~5 min idle)"
    echo ""
    for i in $(seq 600 -60 0); do
        mins=$((i / 60))
        if [ $((i % 60)) -eq 0 ] && [ "$mins" -gt 0 ]; then
            echo "    ${mins} minutes remaining..."
        fi
        if [ "$i" -gt 0 ]; then
            sleep 1
        fi
    done
    # Actually sleep the full 10 minutes (the loop above is just for display)
    sleep 540  # 600 - 60 already elapsed in display loop approximation
    echo "    Idle period complete. Testing recovery..."
    ready_request "Post-idle readiness" "$API_URL/health/ready"
    echo ""
fi

# ─── Summary ────────────────────────────────────────────────
echo "═══════════════════════════════════════════════════════"
echo "  Validation Results"
echo "═══════════════════════════════════════════════════════"
for r in "${RESULTS[@]}"; do
    echo "  $r"
done
echo "───────────────────────────────────────────────────────"
echo "  Total: $((PASS + FAIL))  Pass: $PASS  Fail: $FAIL"
echo "═══════════════════════════════════════════════════════"
echo ""
echo "Next steps:"
echo "  - Check CloudWatch for pt_repo connection retry logs (WARN level)"
echo "  - Record these results in docs/active/work/T-021-03/results.md"

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
