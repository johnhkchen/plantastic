# T-019-02 Design — Dev Stack Recipes

## Decision 1: Handle overlap with existing Docker recipes

### Options
A. **Replace** `up`/`down`/`down-clean` with `dev-db`/`dev-down`/`dev-reset`.
B. **Keep both** — old recipes stay, new ones added alongside.
C. **Rename** old recipes to `dev-*` names, remove originals.

### Decision: C — Rename to `dev-*` namespace
The existing `up`/`down`/`down-clean` are thin wrappers that T-019-01 delivered. The ticket explicitly requests `dev-db`, `dev-down`, `dev-reset` names. Keeping both creates confusion ("which do I use?"). Renaming unifies under the `dev-*` namespace which is clearer: all local development recipes start with `dev-`.

Remove `up`, `down`, `down-clean`. Replace with the ticket's requested names, which are more descriptive and have added functionality (health wait, connection string output).

## Decision 2: `dev-db` implementation — wait strategy

### Options
A. `docker compose up -d db --wait` — native Compose v2 flag, blocks until health check passes.
B. Manual poll loop with `docker inspect`.
C. `docker compose up db` (foreground, no `-d`) — blocks naturally but ties up terminal.

### Decision: A — `--wait` flag
Clean, no custom scripting, works with Compose v2 (standard since Docker Desktop 4.x). Falls back gracefully — if health check passes instantly, no delay. Prints connection string after.

## Decision 3: `dev-stack` approach

### Options
A. Single recipe that backgrounds everything (complex, fragile process management).
B. Print instructions for which recipes to run in separate terminals.
C. Use a process manager (overmind, foreman) — adds a dependency.

### Decision: B — Print instructions with optional background hint
Following the existing `dev` recipe pattern. The `dev-stack` recipe starts compose services (blocking until healthy), then prints the commands for API/web/worker. This is honest — multi-service development needs multiple terminals or a tmux setup. Trying to background everything in one recipe creates zombie processes and hard-to-debug output interleaving.

Enhancement: also offer a one-liner using `&` for users who want background processes, but make the multi-terminal approach the primary recommendation.

## Decision 4: `.env.example` updates

Add missing variables to existing `.env.example`:
- `TEST_DATABASE_URL=postgres://plantastic:plantastic@localhost:5432/plantastic_test` — separate DB for tests
- `VALKEY_URL=redis://localhost:6379` — Valkey is Redis-compatible, `redis://` scheme works
- `S3_ENDPOINT=http://localhost:4566` — placeholder for LocalStack/R2 local (future)
- `RUST_LOG=info` — default log level

Keep existing variables. Organize with section headers for clarity.

## Decision 5: Compose `.env` integration

Make Compose use env vars with defaults via `${VAR:-default}` syntax. This lets `.env` override without breaking the zero-config experience. Variables:
- `POSTGRES_DB=${PLANTASTIC_DB:-plantastic}`
- `POSTGRES_USER=${PLANTASTIC_DB_USER:-plantastic}`
- `POSTGRES_PASSWORD=${PLANTASTIC_DB_PASSWORD:-plantastic}`

Actually — this adds complexity for no real gain. The Compose credentials are local-only, never production. Hardcoding `plantastic/plantastic/plantastic` is fine. The `.env` file's `DATABASE_URL` already encodes these. Adding variable substitution to Compose just creates a second place to misconfigure.

**Revised decision**: Keep Compose credentials hardcoded. The `.env` overrides doc in the ticket means Compose respects `.env` for _any_ env vars it references — which it already does by default (Compose auto-loads `.env`). No changes needed to Compose environment block.

## Decision 6: docker-compose.yml comment block

Add a descriptive comment block at the top explaining:
- What services are included and why
- How to use (reference `just` recipes)
- Where migrations come from
- What ports are exposed

## Decision 7: `dev` recipe disposition

The existing `dev` recipe prints instructions. `dev-stack` supersedes it. Remove `dev` and replace with `dev-stack` to avoid confusion.

## Non-goals
- No process manager (overmind/foreman) — adds dependency, overkill for 3-4 services
- No Compose profiles — premature; two services don't need profiles
- No `dev-worker` recipe yet — worker crate may not be runnable; `dev-stack` will mention it in instructions
