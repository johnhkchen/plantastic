# Plantastic — development commands
# Run `just` with no arguments to see this list.

# Default: show available recipes
default:
    @just --list

# ─── Quality Gates ──────────────────────────────────────────────
# `just check` is the full pre-commit gate. If this passes, your
# work is ready for review. If it doesn't, it isn't.

# Run all quality gates: format, lint, test, scenarios
# NOTE: Full validation also requires `just dev-db && just test-integration`
check: fmt-check lint test scenarios
    @echo ""
    @echo "All gates passed."

# ─── Formatting ─────────────────────────────────────────────────

# Check formatting (fails if anything is unformatted)
fmt-check:
    cargo fmt --all -- --check

# Auto-format all Rust code
fmt:
    cargo fmt --all

# ─── Linting ────────────────────────────────────────────────────

# Run clippy — warnings are errors, targeted pedantic lints enabled
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Run clippy but allow warnings (for development iteration)
lint-warn:
    cargo clippy --workspace --all-targets

# ─── Testing ────────────────────────────────────────────────────

# Run all workspace tests. 60s hard timeout per test binary catches hangs.
# Individual tests should use pt_test_utils::timed() to enforce 10s per test.
# If a Rust test takes >10s, it is almost certainly I/O-blocked, not compute-bound.
test:
    #!/usr/bin/env bash
    set -euo pipefail
    # Compile first so timeout only covers execution
    cargo test --workspace --no-run 2>&1 | tail -1
    echo "Running tests..."
    # Use GNU timeout if available (Linux: timeout, macOS: gtimeout via coreutils)
    TIMEOUT_CMD=""
    if command -v timeout &>/dev/null; then
        TIMEOUT_CMD="timeout 120"
    elif command -v gtimeout &>/dev/null; then
        TIMEOUT_CMD="gtimeout 120"
    fi
    $TIMEOUT_CMD cargo test --workspace 2>&1
    exit_code=${PIPESTATUS[0]:-$?}
    if [ "$exit_code" -ne 0 ]; then
        echo ""
        echo "Tests failed (exit $exit_code)."
        exit "$exit_code"
    fi
    echo ""
    echo "All tests passed."

# Run tests for a specific crate
test-crate crate:
    cargo test -p {{crate}}

# Run integration tests (requires Postgres — start with `just dev-db`)
test-integration:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Running integration tests (requires Postgres)..."
    DATABASE_URL="${DATABASE_URL:-postgres://plantastic:plantastic@localhost:5432/plantastic}" \
        cargo test -p pt-repo --features integration 2>&1
    echo ""
    echo "Integration tests passed."

# Run all integration tests for CI (requires DATABASE_URL env var)
# Runs pt-repo (feature-gated) + plantastic-api (ignored, Postgres-only)
test-integration-ci:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Running pt-repo integration tests..."
    cargo test -p pt-repo --features integration 2>&1
    echo ""
    echo "Running plantastic-api integration tests (Postgres-only)..."
    cargo test -p plantastic-api -- --ignored --skip scan_ 2>&1
    echo ""
    echo "All integration tests passed."

# Run tests and show output (including passing tests)
test-verbose:
    cargo test --workspace -- --nocapture

# ─── Scenarios ──────────────────────────────────────────────────

# Run the value delivery dashboard — the honest scoreboard.
# This is the number that matters. Green unit tests are table stakes.
scenarios:
    @cargo run -p pt-scenarios 2>/dev/null

# ─── Scan Processing ───────────────────────────────────

# Process a PLY scan through the full pipeline (release mode for realistic timing)
process-scan path="assets/scans/samples/Scan at 09.23.ply":
    cargo run -p pt-scan --example process_sample --release -- "{{path}}"

# Process a PLY scan and print instructions to view in the 3D viewer
scan-to-viewer path="assets/scans/samples/Scan at 09.23.ply":
    #!/usr/bin/env bash
    set -euo pipefail
    just process-scan "{{path}}"
    DIR=$(dirname "{{path}}")
    STEM=$(basename "{{path}}" .ply)
    GLB="${DIR}/${STEM}-terrain.glb"
    echo ""
    echo "── Viewer Instructions ──────────────────────────────"
    echo "Terrain GLB: ${GLB}"
    echo ""
    echo "1. Serve the output directory:"
    echo "   python3 -m http.server 8080 -d \"${DIR}\""
    echo ""
    echo "2. Load in the Bevy viewer via postMessage:"
    echo "   { \"type\": \"loadScene\", \"url\": \"http://localhost:8080/${STEM}-terrain.glb\" }"

# ─── Build ──────────────────────────────────────────────────────

# Build the entire workspace
build:
    cargo build --workspace

# Build in release mode
build-release:
    cargo build --workspace --release

# Build the API for Lambda (cross-compile for aarch64-linux via cargo-zigbuild)
build-lambda:
    ./scripts/build-lambda.sh

# ─── Docker / Dev Environment ──────────────────────────────────

# Start database + cache, wait for health, print connection info
dev-db:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Starting Postgres + Valkey..."
    docker compose up -d --wait
    echo ""
    echo "Services healthy."
    echo "  DATABASE_URL=postgres://plantastic:plantastic@localhost:5432/plantastic"
    echo "  VALKEY_URL=redis://localhost:6379"

# Start infra + print commands for API / web / worker
dev-stack:
    #!/usr/bin/env bash
    set -euo pipefail
    just dev-db
    echo ""
    echo "Infrastructure ready. Run these in separate terminals:"
    echo "  just dev-api       # Rust API on :3000"
    echo "  just dev-web       # SvelteKit on :5173"
    echo ""
    echo "Stop everything with: just dev-down"

# Stop all Compose services
dev-down:
    docker compose down

# Nuke volumes and restart fresh (clean slate)
dev-reset:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Removing volumes and restarting..."
    docker compose down -v
    docker compose up -d --wait
    echo ""
    echo "Clean slate ready."
    echo "  DATABASE_URL=postgres://plantastic:plantastic@localhost:5432/plantastic"
    echo "  VALKEY_URL=redis://localhost:6379"

# ─── Database ───────────────────────────────────────────────────

# Run migrations via Doppler (injects DATABASE_URL from Doppler dev config)
migrate:
    doppler run -- ./scripts/migrate.sh

# Run migrations using DATABASE_URL from environment (sqlx-cli)
migrate-direct:
    ./scripts/migrate.sh

# Run database migrations via psql (requires DATABASE_URL env var)
db-migrate:
    @echo "Applying migrations..."
    @for f in $(ls migrations/*.sql | grep -v '\.down\.sql' | sort); do \
        echo "  → $f"; \
        psql "$DATABASE_URL" -f "$f"; \
    done
    @echo "Migrations applied."

# Reset database (drop and recreate)
db-reset:
    @echo "This will destroy all data. Press Ctrl-C to abort."
    @sleep 3
    psql "$DATABASE_URL" -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public; CREATE EXTENSION IF NOT EXISTS postgis;"
    just db-migrate

# ─── Development ────────────────────────────────────────────────

# Run the API locally
dev-api:
    cargo run -p plantastic-api

# Run the SvelteKit frontend dev server
dev-web:
    cd web && npm run dev

# ─── Deployment ─────────────────────────────────────────────────

# Deploy the full stack (build Lambda, SST deploy, CF Pages, CF Worker)
deploy stage="dev":
    @echo "Building Lambda..."
    ./scripts/build-lambda.sh
    @echo "Deploying infrastructure (SST)..."
    cd infra && npx sst deploy --stage {{stage}}
    @echo "Building web..."
    cd web && pnpm run build
    @echo "Deploying web to CF Pages..."
    cd web && npx wrangler pages deploy .svelte-kit/cloudflare --project-name plantastic
    @echo "Deploying CF Worker..."
    cd worker && npx wrangler deploy
    @echo "Deployed to {{stage}}."

# Run smoke tests against a deployed environment
smoke url:
    ./scripts/verify-deploy.sh {{url}}

# Scene generation smoke test (requires running API + Postgres)
# Creates a project with zones + materials, then fetches a scene to verify the pipeline.
smoke-scene url="http://localhost:3000":
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Scene smoke test against {{url}}..."
    TENANT="00000000-0000-0000-0000-000000000001"
    # Create a project
    PROJECT=$(curl -sf -X POST "{{url}}/projects" \
        -H "Content-Type: application/json" \
        -H "X-Tenant-Id: $TENANT" \
        -d '{"client_name":"Smoke Test","address":"123 Test St"}' | jq -r '.id')
    echo "  Created project: $PROJECT"
    # Add a zone
    curl -sf -X POST "{{url}}/projects/$PROJECT/zones" \
        -H "Content-Type: application/json" \
        -H "X-Tenant-Id: $TENANT" \
        -d '[{"geometry":{"type":"Polygon","coordinates":[[[0,0],[12,0],[12,15],[0,15],[0,0]]]},"zone_type":"patio","label":"Test Patio"}]' > /dev/null
    echo "  Added zone"
    # Create a material
    MAT=$(curl -sf -X POST "{{url}}/materials" \
        -H "Content-Type: application/json" \
        -H "X-Tenant-Id: $TENANT" \
        -d '{"name":"Test Paver","category":"hardscape","unit":"sq_ft","price_per_unit":"8.50","extrusion":{"type":"sits_on_top","height_inches":1.5}}' | jq -r '.id')
    echo "  Created material: $MAT"
    # Assign material to tier
    ZONES=$(curl -sf "{{url}}/projects/$PROJECT/zones" -H "X-Tenant-Id: $TENANT" | jq -r '.[0].id')
    curl -sf -X PUT "{{url}}/projects/$PROJECT/tiers/good" \
        -H "Content-Type: application/json" \
        -H "X-Tenant-Id: $TENANT" \
        -d "[{\"zone_id\":\"$ZONES\",\"material_id\":\"$MAT\"}]" > /dev/null
    echo "  Assigned material to tier"
    # Fetch scene
    SCENE=$(curl -sf "{{url}}/projects/$PROJECT/scene/good" -H "X-Tenant-Id: $TENANT")
    URL=$(echo "$SCENE" | jq -r '.url')
    ZONES_COUNT=$(echo "$SCENE" | jq -r '.metadata.zone_count')
    TRIANGLES=$(echo "$SCENE" | jq -r '.metadata.triangle_count')
    echo "  Scene generated: $ZONES_COUNT zones, $TRIANGLES triangles"
    echo "  URL: ${URL:0:80}..."
    # Verify GLB is downloadable
    STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$URL")
    if [ "$STATUS" != "200" ]; then
        echo "  FAIL: GLB download returned $STATUS"
        exit 1
    fi
    echo "  GLB download OK"
    # Cleanup
    curl -sf -X DELETE "{{url}}/projects/$PROJECT" -H "X-Tenant-Id: $TENANT" > /dev/null
    echo "Scene smoke test passed."

# Verify Neon database setup (requires direct + pooled connection URLs)
verify-neon direct pooled:
    ./scripts/verify-neon.sh {{direct}} {{pooled}}

# Validate Lambda → Neon connection (cold starts, concurrency, timing)
validate-neon-lambda url:
    ./scripts/validate-lambda-neon.sh {{url}}

# ─── Lisa / Workflow ────────────────────────────────────────────

# Show ticket DAG status
status:
    lisa status

# ─── Housekeeping ───────────────────────────────────────────────

# Clean all build artifacts
clean:
    cargo clean
    rm -rf web/.svelte-kit web/node_modules/.vite
