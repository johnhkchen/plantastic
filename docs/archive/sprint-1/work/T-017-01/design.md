# T-017-01 Design: Rust Lambda Deploy

## Decision: cargo-zigbuild for Cross-Compilation

### Options Evaluated

**Option A: cargo-zigbuild** (CHOSEN)
- Uses Zig's C compiler as a cross-linker — same approach as HMW Workshop (Go + zig cc)
- No Docker dependency. Fast native builds on macOS.
- Works on both Intel and Apple Silicon Macs.
- Since this project uses `rustls` (no OpenSSL), there are no native C library complications.
- Install: `cargo install cargo-zigbuild` + `brew install zig` (or `pip install ziglang`)
- The ticket explicitly states this is preferred.

**Option B: cross (Docker-based)** — Rejected
- Requires Docker running, which is heavier for local dev and CI.
- Slower due to container startup and volume mounting.
- More hermetic, but unnecessary here since we have no native C deps (rustls, not OpenSSL).
- Would be the right choice if we had OpenSSL or other system library dependencies.

**Option C: Native gcc cross-linker** — Rejected
- Fragile on macOS. Requires `brew install` of aarch64 gcc, `.cargo/config.toml` linker config.
- Poor developer experience. Breaks on toolchain updates.
- No advantage over zigbuild.

### Rationale

cargo-zigbuild is the right tool because:
1. No Docker overhead — faster builds, simpler CI setup
2. Our Rust deps are pure Rust + rustls — no system C libraries to cross-compile
3. Matches the team's proven pattern from HMW Workshop
4. The ticket explicitly prefers it

## Build Script Design

### scripts/build-lambda.sh

Responsibilities:
1. Verify prerequisites (cargo-zigbuild installed, rustup target added)
2. Run `cargo zigbuild` targeting `aarch64-unknown-linux-gnu` in release mode
3. Copy the compiled binary to `target/lambda/plantastic-api/bootstrap`
4. Report binary size

The script should be idempotent and fast to re-run. No Docker, no containers.

### Binary Staging

SST expects: `target/lambda/plantastic-api/bootstrap`

The build script will:
1. Build to the standard cargo output: `target/aarch64-unknown-linux-gnu/release/plantastic-api`
2. Create `target/lambda/plantastic-api/` directory
3. Copy and rename: `plantastic-api` → `bootstrap`

This two-step approach avoids fighting cargo's output paths. The `target/lambda/` directory is ephemeral build output — it goes in `.gitignore`.

## SST Configuration

### Current State (mostly complete)

The SST config at `infra/sst.config.ts` is already well-configured. Key items:
- Lambda function with correct runtime, architecture, handler, bundle path
- Function URL with RESPONSE_STREAM for SSE support
- DatabaseUrl from SSM secret

### What Needs Adding

1. **S3 Bucket**: The SST config doesn't create or reference an S3 bucket. The API expects `S3_BUCKET` env var. For now, we can add it as a hardcoded env var pointing to a pre-existing bucket, or create one via SST. Decision: create an S3 bucket in SST and pass its name. This keeps infra self-contained.

2. **S3_BUCKET env var**: Add to the Lambda function's environment block.

### What Stays the Same
- Memory (256 MB) — conservative starting point, can tune after measuring
- Timeout (30s) — reasonable for API requests
- Region (us-west-2) — matches Neon database location

## Justfile Updates

Replace the current `build-lambda` recipe:
```
# Current (broken on macOS):
build-lambda:
    cargo build -p plantastic-api --release --target aarch64-unknown-linux-gnu

# New:
build-lambda:
    ./scripts/build-lambda.sh
```

## infra/package.json

SST v3 requires npm. Create a minimal `package.json` in `infra/` with:
- `sst` as a dev dependency (latest v3)
- A `deploy` script for convenience

Actually, checking the deploy recipe: `npx sst deploy --stage {{stage}}` uses `npx` which can auto-install. But for reproducibility, a `package.json` with a pinned SST version is better. We'll create one and run `npm install` to generate a lockfile.

## Cold Start Considerations

Rust Lambda cold starts are typically 50-200ms for the runtime itself. The main risk factors:
1. **Binary size** — larger binaries take longer to load. We'll measure and report.
2. **Database pool init** — `pt_repo::create_pool()` establishes connections at startup. If the DB is in a different region or requires TLS handshake, this could add 1-3s.
3. **S3 client init** — `aws-config` SDK initialization involves IMDS calls (~100ms).

Mitigations (if needed, documented but not implemented unless cold start > 5s):
- Use `sqlx` lazy pool (don't connect until first query)
- Reduce binary size with `opt-level = "s"` or `strip = true` in release profile
- Pre-warm with scheduled CloudWatch events

## What This Ticket Does NOT Cover

- Railway/Neon database setup (T-017-02)
- S3 secrets wiring (T-017-02)
- CI/CD pipeline (T-018-01)
- Production deployment (future ticket)

## Scenarios

This ticket doesn't directly flip any scenario to green — it's infrastructure. But it unblocks all scenarios that require a deployed API (S.4.x, S.5.x, S.6.x). The milestone in `tests/scenarios/src/progress.rs` should be claimed after implementation.
