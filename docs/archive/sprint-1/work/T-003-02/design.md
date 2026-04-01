# T-003-02 Design: sqlx Repository Layer

## Decision 1: Crate Location

### Options
**A) Module inside future apps/api crate**: Tight coupling, blocks T-004-01 from starting independently.
**B) New crate `crates/pt-repo`**: Clean boundary, reusable by API and any future consumers (CLI, workers).
**C) Add repo functions directly to pt-project/pt-materials**: Violates single-responsibility — domain crates shouldn't know about Postgres.

### Decision: Option B — `crates/pt-repo`
The repo crate owns all database interaction. Domain crates stay pure. T-004-01 imports pt-repo for its route handlers. The repo crate depends on pt-project, pt-materials, and sqlx.

## Decision 2: PostGIS Geometry Encoding

### Options
**A) SQL-side GeoJSON conversion**: Use `ST_GeomFromGeoJSON($1)` on insert, `ST_AsGeoJSON(geometry)` on select. Geometry travels as JSON text strings in Rust. Simple, no extra crates.
**B) WKB via geozero crate**: Binary encoding. More efficient on the wire. Requires adding geozero + geozero-core to deps. More complex setup.
**C) Raw coordinate extraction**: Build WKT strings manually. Fragile, error-prone.

### Decision: Option A — SQL-side GeoJSON
- The `geojson` crate is already in workspace deps. We serialize `geo::Polygon<f64>` → `geojson::Geometry` → JSON string → pass to `ST_GeomFromGeoJSON`. Reverse on read.
- Performance is not a concern for this use case (single polygons, not bulk spatial operations).
- No new dependencies. The conversion code already exists in pt-project's serde_helpers.
- Trade-off: slightly more SQL boilerplate, but zero new crate dependencies and well-understood behavior.

## Decision 3: Enum String Mapping

### Options
**A) Implement `sqlx::Type` / `sqlx::Encode` / `sqlx::Decode`** for ZoneType, ProjectStatus, etc.: Automatic mapping but requires adding sqlx dependency to domain crates.
**B) Manual string conversion in repo layer**: `impl From<ZoneType> for &str` and `impl TryFrom<&str> for ZoneType` in pt-repo. Keeps domain crates sqlx-free.
**C) Use sqlx `query_as!` with string columns and convert manually in the mapping functions.

### Decision: Option C — string columns with manual conversion in row mapping
- Domain crates already have serde snake_case. The DB stores the same strings. We query as `String`, convert to enum via a helper. This keeps domain crates free of sqlx dependencies.
- We define private `fn parse_zone_type(s: &str) -> Result<ZoneType>` etc. in the repo crate. These are thin wrappers around serde or match statements.
- The repo layer owns all DB-to-domain translation.

## Decision 4: Row Types vs Direct Domain Types

### Decision: Define `*Row` structs in pt-repo for DB mapping
The DB schema has fields the domain types don't (tenant_id, client_name, sort_order, etc.). Rather than polluting domain types with DB concerns, pt-repo defines lightweight row structs:
- `ProjectRow` — flat struct matching the projects table columns
- `ZoneRow` — flat struct matching zones table columns
- `MaterialRow` — flat struct matching materials table columns
- `TierAssignmentRow` — flat struct matching tier_assignments table columns

Each row struct has `fn into_domain(self) -> Result<DomainType>` and corresponding `fn from_domain(domain, extra_fields) -> Row` methods. This is the translation boundary.

## Decision 5: sqlx Query Mode

### Options
**A) Compile-time checked queries (`query!`/`query_as!`)**: Catches SQL errors at compile time. Requires live DB or offline sqlx-data.json.
**B) Runtime queries (`query`/`query_as`)**: No compile-time DB needed. Errors surface at runtime.

### Decision: Option B — runtime queries with `query_as` for now
- The project has no CI database yet. Compile-time checking would block every `cargo build` without a running Postgres.
- We use `sqlx::query_as::<_, Row>()` with explicit bind parameters. Integration tests verify correctness.
- When CI Postgres is available (likely T-004-01 or a CI ticket), we can switch to `query!` macros with `sqlx-data.json` offline mode. This is a tactical decision, not a permanent one.
- All queries are written as string constants at the top of each repo module for easy audit.

## Decision 6: Connection Pool Configuration

### Decision: Expose a `db_pool()` function
```rust
pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(5)
        .min_connections(0)
        .idle_timeout(Duration::from_secs(30))
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await
}
```
Lambda-optimized: small pool, no minimum connections (scale to zero), short idle timeout (Lambda freezes between invocations), aggressive acquire timeout (fail fast rather than queue in Lambda's limited execution time).

## Decision 7: Error Handling

### Decision: Define `RepoError` enum in pt-repo
```rust
pub enum RepoError {
    NotFound,
    Conflict(String),   // unique constraint violations
    Database(sqlx::Error),
    Conversion(String), // domain ↔ row mapping failures
}
```
This wraps sqlx errors and provides domain-meaningful variants. The API layer maps RepoError → HTTP status codes.

## Decision 8: Integration Test Setup

### Decision: Docker-based test Postgres
- Tests use a helper `test_pool()` that connects to `DATABASE_URL` env var (defaulting to `postgres://localhost:5432/plantastic_test`).
- A `setup_test_db()` function runs all migrations on the test DB before tests.
- Tests that need isolation use transactions that roll back (no cross-test contamination).
- `#[ignore]` with scenario ID for tests that need Postgres — they run in CI but not in quick local builds without a DB.

## What Was Rejected

- **geozero for WKB**: Unnecessary complexity for single-polygon operations. Re-evaluate if we ever need bulk spatial queries from Rust.
- **sqlx compile-time checking**: Premature given no CI database. Will adopt when infrastructure supports it.
- **Adding sqlx types to domain crates**: Violates separation. The repo crate is the translation layer.
- **ORM (sea-orm, diesel)**: Overkill. The schema is small and stable. Raw sqlx queries are more transparent and easier to optimize.
