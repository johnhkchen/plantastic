#!/usr/bin/env bash
set -euo pipefail

# Build the Plantastic API for AWS Lambda (aarch64-linux, custom runtime).
# Produces: target/lambda/plantastic-api/bootstrap

TARGET="aarch64-unknown-linux-gnu"
PACKAGE="plantastic-api"
OUT_DIR="target/lambda/${PACKAGE}"

# ── Prerequisites ───────────────────────────────────────────────
if ! command -v cargo-zigbuild &>/dev/null; then
    echo "ERROR: cargo-zigbuild not found."
    echo "  Install: cargo install cargo-zigbuild && brew install zig"
    exit 1
fi

# Ensure the rustup target is installed
if ! rustup target list --installed | grep -q "${TARGET}"; then
    echo "Adding rustup target ${TARGET}..."
    rustup target add "${TARGET}"
fi

# ── Build ───────────────────────────────────────────────────────
echo "Building ${PACKAGE} for ${TARGET} (release)..."
cargo zigbuild -p "${PACKAGE}" --release --target "${TARGET}"

# ── Stage for SST ───────────────────────────────────────────────
mkdir -p "${OUT_DIR}"
cp "target/${TARGET}/release/${PACKAGE}" "${OUT_DIR}/bootstrap"

# ── Report ──────────────────────────────────────────────────────
SIZE=$(du -h "${OUT_DIR}/bootstrap" | cut -f1)
echo ""
echo "Lambda binary ready:"
echo "  Path: ${OUT_DIR}/bootstrap"
echo "  Size: ${SIZE}"
echo "  Target: ${TARGET}"
echo ""
echo "Deploy with: npx sst deploy --stage dev"
