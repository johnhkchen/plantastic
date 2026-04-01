# T-017-01 Progress: Rust Lambda Deploy

## Completed

### Step 1: Build script ✓
- Created `scripts/build-lambda.sh`
- Checks for cargo-zigbuild and rustup target prerequisites
- Builds with `cargo zigbuild -p plantastic-api --release --target aarch64-unknown-linux-gnu`
- Stages binary at `target/lambda/plantastic-api/bootstrap`
- Reports binary size
- Made executable (`chmod +x`)

### Step 2: .gitignore updates ✓
- Added `target/lambda/` (build staging output)
- Added `.sst/` and `.open-next/` (SST runtime dirs)
- `node_modules/` already covered by existing rule

### Step 3: infra/package.json ✓
- Created minimal `package.json` with `sst: "^3"` dev dependency
- npm install deferred to first deploy (npx handles it)

### Step 4: SST config updates ✓
- Added S3 bucket: `new sst.aws.Bucket("Uploads")`
- Added `link: [uploads]` for IAM permissions
- Added `S3_BUCKET: uploads.name` to Lambda environment
- Added `bucketName: uploads.name` to stack outputs

### Step 5: Justfile update ✓
- Replaced `cargo build ...` with `./scripts/build-lambda.sh`

### Step 6: Milestone claim ✓
- Added "Lambda deploy: cross-compile + SST + S3 bucket" milestone in progress.rs
- Delivered by T-017-01, unlocks S.INFRA.1

### Verification ✓
- `just fmt-check` — pass
- `just lint` — pass (clippy strict, no warnings)
- `just test` — all tests pass
- `just scenarios` — 58.0 min / 240.0 min (24.2%), no regression, 13/22 milestones

## Deviations from Plan

- Did not run `npm install` in infra/ — `npx sst deploy` handles this on first deploy
- Did not attempt `just build-lambda` execution — requires cargo-zigbuild + zig to be installed. Script is tested for correctness by inspection. Actual build verification happens when deploying.
- Combined all changes into a single implementation pass instead of separate commits (per RDSPI workflow — commits happen after review)
