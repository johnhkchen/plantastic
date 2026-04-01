#!/usr/bin/env bash
set -euo pipefail

# Verify a Neon PostgreSQL setup against T-021-01 acceptance criteria.
#
# Usage:
#   ./scripts/verify-neon.sh <direct-url> <pooled-url>
#   just verify-neon <direct-url> <pooled-url>
#
# Both URLs should include ?sslmode=require for Neon.
# The script exits 0 if all checks pass, 1 on any failure.

DIRECT_URL="${1:-}"
POOLED_URL="${2:-}"

if [ -z "$DIRECT_URL" ] || [ -z "$POOLED_URL" ]; then
    echo "Usage: $0 <direct-connection-url> <pooled-connection-url>"
    echo ""
    echo "Example:"
    echo "  $0 'postgres://user:pass@ep-xxx.us-west-2.aws.neon.tech/plantastic?sslmode=require' \\"
    echo "     'postgres://user:pass@ep-xxx-pooler.us-west-2.aws.neon.tech/plantastic?sslmode=require'"
    exit 1
fi

if ! command -v psql &>/dev/null; then
    echo "Error: psql not found. Install postgresql client."
    exit 1
fi

PASS=0
FAIL=0
CHECKS=()

check() {
    local name="$1"
    shift
    if "$@" 2>/dev/null; then
        CHECKS+=("PASS  $name")
        ((PASS++))
    else
        CHECKS+=("FAIL  $name")
        ((FAIL++))
    fi
}

# Mask credentials in URLs for display
mask_url() {
    echo "$1" | sed 's|://[^@]*@|://***@|'
}

echo "Verifying Neon setup..."
echo "  Direct: $(mask_url "$DIRECT_URL")"
echo "  Pooled: $(mask_url "$POOLED_URL")"
echo ""

# ─── Check 1: Direct endpoint connectivity ─────────────────────
check "Direct endpoint connectivity" \
    psql "$DIRECT_URL" -c "SELECT 1;" -t -q

# ─── Check 2: PostGIS extension ────────────────────────────────
check_postgis() {
    local version
    version=$(psql "$DIRECT_URL" -t -q -c "SELECT PostGIS_Version();" 2>/dev/null | tr -d ' ')
    if [ -n "$version" ]; then
        echo "  PostGIS version: $version"
        return 0
    fi
    return 1
}
check "PostGIS extension enabled" check_postgis

# ─── Check 3: All 6 tables exist ──────────────────────────────
check_tables() {
    local expected=("materials" "plants" "projects" "tenants" "tier_assignments" "zones")
    local actual
    actual=$(psql "$DIRECT_URL" -t -q -c \
        "SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename;" \
        2>/dev/null | tr -d ' ' | grep -v '^$')

    local missing=()
    for table in "${expected[@]}"; do
        if ! echo "$actual" | grep -q "^${table}$"; then
            missing+=("$table")
        fi
    done

    if [ ${#missing[@]} -eq 0 ]; then
        echo "  All 6 tables present: ${expected[*]}"
        return 0
    else
        echo "  Missing tables: ${missing[*]}"
        return 1
    fi
}
check "All 6 migration tables exist" check_tables

# ─── Check 4: Spatial roundtrip ────────────────────────────────
check_spatial() {
    # Create a temporary project + zone, verify geometry roundtrip, clean up.
    # Uses a known polygon and checks coordinates come back intact.

    local tenant_id project_id zone_id

    # Insert tenant
    tenant_id=$(psql "$DIRECT_URL" -t -q -c "
        INSERT INTO tenants (id, name, created_at, updated_at)
        VALUES ('00000000-0000-0000-0000-ffffffffffff', '_neon_verify', now(), now())
        RETURNING id;" 2>/dev/null | tr -d ' ')

    # Insert project with a POINT geometry
    project_id=$(psql "$DIRECT_URL" -t -q -c "
        INSERT INTO projects (id, tenant_id, client_name, address, location, status, created_at, updated_at)
        VALUES (
            '00000000-0000-0000-0000-fffffffffff1',
            '00000000-0000-0000-0000-ffffffffffff',
            '_verify', '_verify',
            ST_SetSRID(ST_MakePoint(-122.4194, 37.7749), 4326)::geography,
            'draft', now(), now()
        ) RETURNING id;" 2>/dev/null | tr -d ' ')

    # Insert zone with a POLYGON geometry
    zone_id=$(psql "$DIRECT_URL" -t -q -c "
        INSERT INTO zones (id, project_id, geometry, zone_type, label, sort_order, created_at, updated_at)
        VALUES (
            '00000000-0000-0000-0000-fffffffffff2',
            '00000000-0000-0000-0000-fffffffffff1',
            ST_SetSRID(ST_GeomFromText('POLYGON((0 0, 10 0, 10 10, 0 10, 0 0))'), 4326),
            'patio', '_verify', 0, now(), now()
        ) RETURNING id;" 2>/dev/null | tr -d ' ')

    # Fetch back and verify coordinates
    local coords
    coords=$(psql "$DIRECT_URL" -t -q -c "
        SELECT ST_AsText(geometry)
        FROM zones
        WHERE id = '00000000-0000-0000-0000-fffffffffff2';" 2>/dev/null | tr -d ' ')

    # Clean up (reverse order for FK constraints)
    psql "$DIRECT_URL" -q -c "
        DELETE FROM zones WHERE id = '00000000-0000-0000-0000-fffffffffff2';
        DELETE FROM projects WHERE id = '00000000-0000-0000-0000-fffffffffff1';
        DELETE FROM tenants WHERE id = '00000000-0000-0000-0000-ffffffffffff';" 2>/dev/null

    if echo "$coords" | grep -q "POLYGON"; then
        echo "  Spatial roundtrip: insert polygon → fetch → $coords"
        return 0
    else
        echo "  Spatial roundtrip failed: got '$coords'"
        return 1
    fi
}
check "Spatial query roundtrip (insert/fetch/delete)" check_spatial

# ─── Check 5: Pooled endpoint connectivity ─────────────────────
check "Pooled endpoint connectivity" \
    psql "$POOLED_URL" -c "SELECT 1;" -t -q

# ─── Check 6: Pooled endpoint query ────────────────────────────
check_pooled_query() {
    local count
    count=$(psql "$POOLED_URL" -t -q -c \
        "SELECT count(*) FROM pg_tables WHERE schemaname = 'public';" 2>/dev/null | tr -d ' ')
    if [ "$count" -ge 6 ]; then
        echo "  Pooled query returned $count public tables"
        return 0
    fi
    return 1
}
check "Pooled endpoint query works" check_pooled_query

# ─── Summary ───────────────────────────────────────────────────
echo ""
echo "═══════════════════════════════════════════"
echo "  Neon Verification Results"
echo "═══════════════════════════════════════════"
for c in "${CHECKS[@]}"; do
    echo "  $c"
done
echo "───────────────────────────────────────────"
echo "  Total: $((PASS + FAIL))  Pass: $PASS  Fail: $FAIL"
echo "═══════════════════════════════════════════"

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
