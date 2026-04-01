# T-017-01 Plan: Rust Lambda Deploy

## Step 1: Create build script

Create `scripts/build-lambda.sh`:
- Check `cargo-zigbuild` is installed (exit with install instructions if not)
- Check `aarch64-unknown-linux-gnu` rustup target is installed (auto-add if missing)
- Run `cargo zigbuild -p plantastic-api --release --target aarch64-unknown-linux-gnu`
- Create `target/lambda/plantastic-api/` directory
- Copy binary to `target/lambda/plantastic-api/bootstrap`
- Print binary size in MB
- `chmod +x`

Verification: script exists and is executable.

## Step 2: Update .gitignore

Add entries for:
- `target/lambda/` — build output staging
- `infra/node_modules/`
- `infra/.sst/`
- `infra/.open-next/`

Check existing .gitignore first to avoid duplicates.

Verification: `git status` doesn't show target/lambda/ as untracked after build.

## Step 3: Create infra/package.json

Create minimal `package.json` with SST v3 as dev dependency.
Run `npm install` in `infra/` to generate lockfile.

Verification: `infra/node_modules/sst` exists.

## Step 4: Update SST config

Modify `infra/sst.config.ts`:
- Add S3 bucket: `new sst.aws.Bucket("Uploads")`
- Add `S3_BUCKET` env var to Lambda function referencing bucket name
- Link bucket to function for IAM permissions

Verification: TypeScript parses without errors.

## Step 5: Update justfile

Replace `build-lambda` recipe body with `./scripts/build-lambda.sh`.

Verification: `just --list` shows the recipe, `just build-lambda` calls the script.

## Step 6: Build verification

Run `just build-lambda` (requires cargo-zigbuild + zig installed).
If tools not installed, document the install commands but don't fail the ticket.

Verification:
- `target/lambda/plantastic-api/bootstrap` exists
- `file target/lambda/plantastic-api/bootstrap` shows aarch64 ELF binary
- Note binary size

## Step 7: Claim milestone

Update `tests/scenarios/src/progress.rs` if a Lambda deployment milestone exists.
If not, add one that documents what this ticket delivers and what it unblocks.

Verification: `just scenarios` runs without regression.

## Testing Strategy

This ticket is infrastructure — no unit tests apply. Verification is:
1. Build script produces correct binary (Step 6)
2. Binary is correct architecture (aarch64 ELF)
3. SST config parses correctly
4. `just check` passes (no regressions in existing tests)
5. Deployment verification (manual — deploy to dev, hit health endpoint) is documented but may not be executable in this session if AWS credentials aren't configured

## Commit Strategy

- Commit 1: build script + .gitignore updates
- Commit 2: infra/package.json + SST config updates
- Commit 3: justfile update
- Commit 4: milestone claim (if applicable)

Each commit is independently valid.
