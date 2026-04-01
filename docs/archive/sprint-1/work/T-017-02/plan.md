# T-017-02 Plan: Implementation Steps

## Step 1: Doppler Configuration

**Create `.doppler.yaml`** in project root with project "plantastic", config "dev".

**Update `.env.example`** with Neon connection string format and Doppler usage note.

**Verification:** File exists, `doppler run -- env | grep DATABASE` would work
(requires Doppler CLI + project setup, which is manual).

---

## Step 2: S3 CORS in SST

**Modify `infra/sst.config.ts`:** Add `cors` property to the `Uploads` bucket with
permissive origins (wildcard for now), GET/PUT/POST methods, all headers, 1 hour max age.

**Verification:** `npx sst diff` shows CORS addition (or visual review of config).

---

## Step 3: Migration Script + Justfile

**Create `scripts/migrate.sh`:**
- Check for `sqlx-cli` installation
- Run `sqlx migrate run --source migrations`
- Report results

**Update `justfile`:** Add `migrate` (via Doppler) and `migrate-direct` recipes.

**Verification:** `just migrate-direct` runs against local Postgres if available.
Script exits cleanly with helpful error if sqlx-cli not installed.

---

## Step 4: Verification Script

**Create `scripts/verify-deploy.sh`:**
- Accept Lambda Function URL as argument
- Test health endpoint (GET /)
- Create project (POST /projects)
- Fetch project (GET /projects/{id})
- Report timing and results

**Verification:** Script is executable, shows usage when called without args.

---

## Step 5: Neon Setup Guide

**Create `docs/active/work/T-017-02/setup-neon.md`:**
Step-by-step instructions for the manual Neon setup:
1. Create Neon project
2. Enable PostGIS
3. Get connection strings
4. Set SST secret
5. Set Doppler secrets
6. Run migrations
7. Verify connectivity

This is documentation, not code — but it's the operational runbook for this ticket.

---

## Step 6: Claim Milestone

**Update `tests/scenarios/src/progress.rs`:**
Add milestone for "Neon PostGIS + S3 CORS + secrets wiring" delivered by T-017-02.
Document what this unblocks (deployed API scenarios, scan pipeline, quoting).

**Verification:** `cargo run -p pt-scenarios` runs without regression.

---

## Step 7: Quality Gate

Run `just check` (format + lint + test + scenarios). Fix any issues.

---

## Testing Strategy

This ticket is infrastructure configuration — no new Rust code, so no new unit or
integration tests. Verification is:

1. **Static:** SST config is valid TypeScript (SST validates on deploy)
2. **Script:** migrate.sh and verify-deploy.sh are functional shell scripts
3. **Scenarios:** `cargo run -p pt-scenarios` shows no regression + new milestone
4. **Quality gate:** `just check` passes
5. **Manual:** Neon setup guide is followed to verify end-to-end connectivity
   (documented in review.md)

No Rust application code changes means `just test` and `just lint` should pass
unchanged. The milestone claim adds to scenario output without breaking anything.
