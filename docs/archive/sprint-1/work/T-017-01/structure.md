# T-017-01 Structure: Rust Lambda Deploy

## Files Created

### scripts/build-lambda.sh (new)
Build script for cross-compiling the API to Lambda's aarch64 target.

```
#!/usr/bin/env bash
set -euo pipefail

# 1. Check prerequisites
#    - cargo-zigbuild installed
#    - aarch64-unknown-linux-gnu target installed via rustup

# 2. Build
#    cargo zigbuild -p plantastic-api --release --target aarch64-unknown-linux-gnu

# 3. Stage for SST
#    mkdir -p target/lambda/plantastic-api
#    cp target/aarch64-unknown-linux-gnu/release/plantastic-api target/lambda/plantastic-api/bootstrap

# 4. Report
#    Print binary size
```

Executable (`chmod +x`). ~30 lines.

### infra/package.json (new)
Minimal npm package for SST v3 dependency.

```json
{
  "name": "plantastic-infra",
  "private": true,
  "devDependencies": {
    "sst": "^3"
  }
}
```

After creation, `cd infra && npm install` generates `package-lock.json`.

## Files Modified

### justfile
- Replace `build-lambda` recipe body: `cargo build ...` → `./scripts/build-lambda.sh`
- No other recipe changes needed

### infra/sst.config.ts
- Add S3 bucket resource: `new sst.aws.Bucket("Uploads")`
- Add `S3_BUCKET` to Lambda environment, referencing the bucket name
- Add `link` to connect bucket to function (grants IAM permissions automatically)

### .gitignore
- Add `target/lambda/` to prevent committing staged build output
- Add `infra/node_modules/` and `infra/.sst/` if not already present

## Files Unchanged

### crates/plantastic-api/src/main.rs
Already handles Lambda mode. No changes needed.

### crates/plantastic-api/Cargo.toml
All dependencies (lambda_http, aws-sdk-s3, etc.) already present.

### Cargo.toml (workspace root)
Workspace structure and deps are correct.

## Module Boundaries

This ticket touches infrastructure only — no Rust code changes. The boundaries are:

```
scripts/build-lambda.sh  ──builds──>  target/lambda/plantastic-api/bootstrap
                                              │
infra/sst.config.ts      ──bundles──>  (reads bootstrap from target/lambda/)
                                              │
justfile                 ──orchestrates──>  build-lambda → scripts/build-lambda.sh
                                            deploy → npx sst deploy
```

## Ordering

1. Create `scripts/build-lambda.sh` (independent)
2. Create `infra/package.json` and install deps (independent)
3. Update `.gitignore` (independent)
4. Update `justfile` (depends on #1 existing)
5. Update `infra/sst.config.ts` (independent)
6. Build and verify locally: `just build-lambda` (depends on #1, #4)
7. Deploy: `just deploy` (depends on #2, #5, #6)

Steps 1-3 and 5 can be done in parallel. Step 4 depends on 1. Steps 6-7 are sequential verification.
