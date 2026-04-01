use crate::registry::{Scenario, ScenarioOutcome, ValueArea};

pub fn scenarios() -> &'static [Scenario] {
    &SCENARIOS
}

static SCENARIOS: [Scenario; 2] = [
    Scenario {
        id: "S.INFRA.1",
        name: "Full stack round-trip",
        area: ValueArea::Infrastructure,
        time_savings_minutes: 0.0,
        replaces: "N/A — infrastructure correctness, not user time savings",
        test_fn: s_infra_1_full_stack,
    },
    Scenario {
        id: "S.INFRA.2",
        name: "Tenant isolation",
        area: ValueArea::Infrastructure,
        time_savings_minutes: 0.0,
        replaces: "N/A — security correctness, not user time savings",
        test_fn: s_infra_2_tenant_isolation,
    },
];

fn s_infra_1_full_stack() -> ScenarioOutcome {
    // TARGET: This should turn green at end of Sprint 1 (T-006-01).
    //
    // When implemented:
    // 1. POST /projects → 201 with project id
    // 2. GET /projects/:id → 200 with matching data
    // 3. POST /projects/:id/zones (known geometry) → 201
    // 4. GET /projects/:id/zones → zone present with correct geometry
    // 5. POST /materials → 201
    // 6. PUT /projects/:id/tiers/good (assign material to zone) → 200
    // 7. GET /projects/:id/quote/good → 200 with correct line items
    // 8. DELETE /projects/:id → 200
    // 9. GET /projects/:id → 404
    //
    // All requests go through the full stack (CF Pages → Worker → Lambda → PostGIS).
    // For CI, can run against local Axum + Postgres directly.

    ScenarioOutcome::NotImplemented
}

fn s_infra_2_tenant_isolation() -> ScenarioOutcome {
    // When implemented:
    // 1. Create project as Tenant A
    // 2. Attempt to fetch as Tenant B → 404 (not 403, don't leak existence)
    // 3. Create material as Tenant A
    // 4. List materials as Tenant B → Tenant A's material not in list
    // 5. Attempt zone/tier operations on Tenant A's project as Tenant B → 404

    ScenarioOutcome::NotImplemented
}
