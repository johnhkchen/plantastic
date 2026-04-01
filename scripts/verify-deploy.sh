#!/usr/bin/env bash
set -euo pipefail

# Verify a deployed Plantastic Lambda API.
# Hits health, creates a project, fetches it back, and reports timing.
#
# Usage: ./scripts/verify-deploy.sh <lambda-function-url>

if [ $# -lt 1 ]; then
    echo "Usage: $0 <api-url>"
    echo "Example: $0 https://xxx.lambda-url.us-west-2.on.aws"
    exit 1
fi

API_URL="${1%/}"
TENANT_ID="tenant-verify-$(date +%s)"
PASS=0
FAIL=0

pass() { echo "  ✓ $1"; PASS=$((PASS + 1)); }
fail() { echo "  ✗ $1"; FAIL=$((FAIL + 1)); }

echo "Verifying deployment at: $API_URL"
echo ""

# 1. Health check
echo "1. Health check (GET /health)"
START=$(date +%s%N)
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$API_URL/health")
END=$(date +%s%N)
ELAPSED=$(( (END - START) / 1000000 ))
if [ "$HTTP_CODE" = "200" ]; then
    pass "Health OK (${ELAPSED}ms — includes cold start if first request)"
else
    fail "Health returned HTTP $HTTP_CODE (${ELAPSED}ms)"
fi

# 2. Create a project
echo "2. Create project (POST /projects)"
CREATE_RESPONSE=$(curl -s -w "\n%{http_code}" \
    -X POST "$API_URL/projects" \
    -H "Content-Type: application/json" \
    -H "X-Tenant-Id: $TENANT_ID" \
    -d '{"name": "Verify Deploy Test", "address": "123 Test St"}')
CREATE_BODY=$(echo "$CREATE_RESPONSE" | head -n -1)
CREATE_CODE=$(echo "$CREATE_RESPONSE" | tail -n 1)

if [ "$CREATE_CODE" = "201" ] || [ "$CREATE_CODE" = "200" ]; then
    PROJECT_ID=$(echo "$CREATE_BODY" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
    if [ -n "$PROJECT_ID" ]; then
        pass "Created project: $PROJECT_ID"
    else
        fail "Created but couldn't parse project ID"
        PROJECT_ID=""
    fi
else
    fail "Create returned HTTP $CREATE_CODE"
    PROJECT_ID=""
fi

# 3. Fetch project back
if [ -n "$PROJECT_ID" ]; then
    echo "3. Fetch project (GET /projects/$PROJECT_ID)"
    FETCH_RESPONSE=$(curl -s -w "\n%{http_code}" \
        -H "X-Tenant-Id: $TENANT_ID" \
        "$API_URL/projects/$PROJECT_ID")
    FETCH_BODY=$(echo "$FETCH_RESPONSE" | head -n -1)
    FETCH_CODE=$(echo "$FETCH_RESPONSE" | tail -n 1)

    if [ "$FETCH_CODE" = "200" ]; then
        if echo "$FETCH_BODY" | grep -q "Verify Deploy Test"; then
            pass "Fetched project with correct name"
        else
            fail "Fetched but name doesn't match"
        fi
    else
        fail "Fetch returned HTTP $FETCH_CODE"
    fi

    # 4. Clean up — delete the test project
    echo "4. Cleanup (DELETE /projects/$PROJECT_ID)"
    DEL_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
        -X DELETE "$API_URL/projects/$PROJECT_ID" \
        -H "X-Tenant-Id: $TENANT_ID")
    if [ "$DEL_CODE" = "200" ] || [ "$DEL_CODE" = "204" ]; then
        pass "Deleted test project"
    else
        fail "Delete returned HTTP $DEL_CODE (non-critical)"
    fi
else
    echo "3. Skipping fetch (no project ID)"
    echo "4. Skipping cleanup (no project ID)"
fi

echo ""
echo "Results: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] && echo "Deployment verified." || echo "Deployment has issues."
exit "$FAIL"
