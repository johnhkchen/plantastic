# Design — T-008-01: Quote API Route

## Decision 1: Route shape

**Options:**
1. `GET /projects/:id/quote/:tier` — single tier per request
2. `GET /projects/:id/quote` — returns all three tiers in one response
3. `GET /projects/:id/tiers/:tier/quote` — nested under existing tiers path

**Decision: Option 1** — `GET /projects/:id/quote/:tier`

Rationale: The AC explicitly specifies this shape. Returning a single tier per request
keeps the response focused and matches how the frontend will use it (select a tier tab →
fetch that tier's quote). Option 2 would require 3x the DB queries per request for a
typical "show one tier" view. Option 3 is semantically fine but deviates from the AC.

## Decision 2: Type conversion location

**Options:**
1. Inline conversion in the handler — transform repo rows to domain types directly
2. Conversion functions in a shared module or on the repo types
3. Conversion functions inside the new `quotes.rs` route module

**Decision: Option 3** — private conversion functions in `quotes.rs`

Rationale: The conversions are specific to this handler's needs (assembling `Zone`,
`Tier`, `Material` from repo rows for `compute_quote`). No other route needs this
exact transformation. Putting them in `quotes.rs` keeps them co-located with the only
consumer and avoids polluting shared modules. If a second consumer appears later,
extract then.

## Decision 3: QuoteError handling

**Options:**
1. `impl From<QuoteError> for AppError` on the AppError type
2. Map inline with `.map_err()` in the handler

**Decision: Option 2** — inline map_err

Rationale: QuoteError has only two variants and both should be 400. An `impl From` would
couple the API error module to pt-quote permanently. The handler knows the semantic
context — a MaterialNotFound means the tenant's catalog data is incomplete, which is a
client-side data issue (400), not an internal error (500). Inline mapping makes this explicit.

## Decision 4: Tier parsing reuse

**Options:**
1. Make `tiers::parse_tier` `pub(crate)` and call it from `quotes.rs`
2. Duplicate the tier parsing logic in `quotes.rs`

**Decision: Option 1** — make parse_tier pub(crate)

Rationale: The function is 5 lines but already has the right error message format. Duplication
means two places to update if we add tiers later. `pub(crate)` is the right visibility.

## Decision 5: Response DTO shape

**Options:**
1. Return `pt_quote::Quote` directly (it derives `Serialize`)
2. Define a `QuoteResponse` DTO that mirrors the domain type
3. Define a `QuoteResponse` with some fields renamed/adjusted

**Decision: Option 1** — return `Quote` directly via `Json<Quote>`

Rationale: `Quote` and `LineItem` already derive `Serialize` with the exact JSON shape we
want: `tier`, `line_items`, `subtotal`, `tax`, `total`. Each `LineItem` includes `zone_id`,
`zone_label`, `material_id`, `material_name`, `quantity`, `unit`, `unit_price`, `line_total`.
This is a clean API response — no reason to add an adapter layer. If the API needs a
different shape in the future, we can add a DTO then.

Note: `ZoneId` and `MaterialId` are newtype wrappers around `Uuid` that serialize as plain
UUIDs, so the JSON output is clean.

## Decision 6: Material loading scope

**Options:**
1. Load only materials referenced by the tier's assignments (N queries or IN clause)
2. Load the full tenant material catalog (one query)

**Decision: Option 2** — load full catalog

Rationale: `material::list_by_tenant` is a single query that returns all materials for the
tenant. The catalog is small (tens to low hundreds of materials per tenant, not thousands).
`compute_quote` does a linear scan `find()` for each assignment — O(assignments × materials)
which is negligible for realistic sizes. A targeted IN-clause query would add complexity
for no measurable benefit.

## Decision 7: Scenario advancement

The ticket advances S.3.1 and S.3.2 from OneStar to TwoStar. However, the scenario
test functions themselves test the pure compute engine, not the API. The correct action is:

1. Do NOT modify the scenario test functions (they're correct as-is).
2. Add the integration test in `plantastic-api/tests/` that exercises the API route.
3. Once the route exists and the integration test passes, the scenario dashboard commentary
   can note API availability, but the scenario outcome stays at OneStar until the test
   itself exercises the API path. The scenario test would need to be updated to call the
   API to claim TwoStar — that's T-008-02's job.

## Rejected alternative: lazy quote computation

Considered computing and caching quotes in the database when assignments change (materialized
view pattern). Rejected because: quotes are cheap to compute (pure arithmetic, no I/O beyond
loading data), caching adds invalidation complexity, and the AC says "compute on request."

## Error response contract

| Condition | HTTP Status | Body |
|-----------|-----------|------|
| Project not found | 404 | `{"error": "not found"}` |
| Invalid tier name | 400 | `{"error": "invalid tier: xyz (expected good, better, or best)"}` |
| Material not in catalog | 400 | `{"error": "material {id} not found in catalog (referenced by zone {id})"}` |
| Missing depth for cu_yd material | 400 | `{"error": "cu_yd material \"name\" (id) has no depth_inches and no override"}` |
| DB error | 500 | `{"error": "internal error"}` |
| No assignments for tier | 200 | `{"tier":"good","line_items":[],"subtotal":"0","tax":null,"total":"0"}` |
