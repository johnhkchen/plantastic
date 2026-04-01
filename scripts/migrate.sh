#!/usr/bin/env bash
set -euo pipefail

# Run database migrations using sqlx-cli.
# DATABASE_URL must be set (directly or via doppler run).
#
# Usage:
#   DATABASE_URL=postgres://... ./scripts/migrate.sh
#   doppler run -- ./scripts/migrate.sh
#   just migrate          # uses Doppler
#   just migrate-direct   # uses DATABASE_URL from env

if ! command -v sqlx &>/dev/null; then
    echo "Error: sqlx-cli not installed."
    echo "Install: cargo install sqlx-cli --no-default-features --features postgres"
    exit 1
fi

if [ -z "${DATABASE_URL:-}" ]; then
    echo "Error: DATABASE_URL not set."
    echo "Set it directly or use: doppler run -- ./scripts/migrate.sh"
    exit 1
fi

echo "Running migrations against: ${DATABASE_URL%%@*}@***"
sqlx migrate run --source migrations
echo "Migrations complete."
