# T-019-02 Plan — Dev Stack Recipes

## Step 1: Add comment block to docker-compose.yml

Add the descriptive comment block at the top of `docker-compose.yml` before `services:`.

**Verify:** File parses correctly — `docker compose config` should succeed.

## Step 2: Update .env.example

Add missing variables (`TEST_DATABASE_URL`, `VALKEY_URL`, `S3_ENDPOINT`, `RUST_LOG`), reorganize with section headers.

**Verify:** All variables from acceptance criteria are present.

## Step 3: Replace justfile Docker/dev recipes

Remove `up`, `down`, `down-clean`, and `dev` recipes. Add `dev-db`, `dev-stack`, `dev-down`, `dev-reset` recipes.

**Verify:** `just --list` shows new recipes, old names absent.

## Step 4: Verify `just` with no args shows recipes

Run `just` and confirm all four new `dev-*` recipes appear in the list with descriptions.

## Testing Strategy

This ticket is infrastructure/DX — no Rust code changes, no unit tests, no scenario impact. Verification is manual:

1. `just --list` shows all four new recipes with descriptions
2. `docker compose config` validates the Compose file still parses
3. `.env.example` contains all required variables
4. Comment block is present at top of docker-compose.yml

No scenario dashboard impact — this is developer tooling, not customer-facing capability.
