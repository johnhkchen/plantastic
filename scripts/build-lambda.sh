#!/usr/bin/env bash
set -euo pipefail

# Build the Plantastic API for AWS Lambda (aarch64-linux, custom runtime).
# Downloads BAML native library and bundles it alongside the binary.
# Produces: target/lambda/plantastic-api/{bootstrap, libbaml_cffi-*.so}

TARGET="aarch64-unknown-linux-gnu"
PACKAGE="plantastic-api"
OUT_DIR="target/lambda/${PACKAGE}"
LIB_FILENAME="libbaml_cffi-aarch64-unknown-linux-gnu.so"

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

# ── Download BAML native library ───────────────────────────────
# Extract BAML version from Cargo.lock (single source of truth)
BAML_VERSION=$(grep -A1 'name = "baml"' Cargo.lock 2>/dev/null | grep 'version' | head -1 | sed 's/.*"\(.*\)".*/\1/' || true)

if [[ -z "${BAML_VERSION}" ]]; then
    # Fallback: try workspace Cargo.toml
    BAML_VERSION=$(grep 'baml.*=' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
fi

if [[ -z "${BAML_VERSION}" ]]; then
    echo "ERROR: Could not determine BAML version from Cargo.lock or generators.baml"
    exit 1
fi

DOWNLOAD_URL="https://github.com/boundaryml/baml/releases/download/${BAML_VERSION}/${LIB_FILENAME}"

download_baml_lib() {
    echo "Downloading BAML native library (v${BAML_VERSION})..."

    # Try downloading with SHA256 verification
    local checksum_url="${DOWNLOAD_URL}.sha256"
    local checksum_file
    checksum_file=$(mktemp)
    trap "rm -f '${checksum_file}'" RETURN

    if curl -fSL --retry 2 -o "${checksum_file}" "${checksum_url}" 2>/dev/null; then
        curl -fSL --retry 2 -o "${OUT_DIR}/${LIB_FILENAME}" "${DOWNLOAD_URL}"
        local expected actual
        expected=$(head -1 "${checksum_file}" | awk '{print $1}')
        actual=$(shasum -a 256 "${OUT_DIR}/${LIB_FILENAME}" | awk '{print $1}')
        if [[ "${expected}" != "${actual}" ]]; then
            echo "  ERROR: Checksum mismatch!"
            echo "    Expected: ${expected}"
            echo "    Actual:   ${actual}"
            rm -f "${OUT_DIR}/${LIB_FILENAME}"
            exit 1
        fi
        echo "  Checksum verified."
    else
        echo "  Warning: checksum file not available, downloading without verification"
        curl -fSL --retry 2 -o "${OUT_DIR}/${LIB_FILENAME}" "${DOWNLOAD_URL}"
    fi

    echo "  Downloaded: ${LIB_FILENAME} ($(du -h "${OUT_DIR}/${LIB_FILENAME}" | cut -f1))"
}

# ── Build ───────────────────────────────────────────────────────
echo "Building ${PACKAGE} for ${TARGET} (release)..."
cargo zigbuild -p "${PACKAGE}" --release --target "${TARGET}"

# ── Stage for SST ───────────────────────────────────────────────
mkdir -p "${OUT_DIR}"
cp "target/${TARGET}/release/${PACKAGE}" "${OUT_DIR}/bootstrap"

# Download BAML .so if not already present
if [[ -f "${OUT_DIR}/${LIB_FILENAME}" ]]; then
    echo "BAML native library already present, skipping download."
else
    download_baml_lib
fi

# ── Report ──────────────────────────────────────────────────────
BOOTSTRAP_SIZE=$(du -h "${OUT_DIR}/bootstrap" | cut -f1)
LIB_SIZE=$(du -h "${OUT_DIR}/${LIB_FILENAME}" 2>/dev/null | cut -f1 || echo "missing")
echo ""
echo "Lambda bundle ready:"
echo "  Path:      ${OUT_DIR}/"
echo "  bootstrap: ${BOOTSTRAP_SIZE}"
echo "  BAML .so:  ${LIB_SIZE}"
echo "  Target:    ${TARGET}"
echo ""
echo "Deploy with: npx sst deploy --stage dev"
