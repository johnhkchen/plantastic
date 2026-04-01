use crate::registry::{Integration, Polish, Scenario, ScenarioOutcome, ValueArea};

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
    if std::env::var("DATABASE_URL").is_err() {
        return ScenarioOutcome::Blocked(
            "no DATABASE_URL — full-stack round-trip requires a real database".to_string(),
        );
    }

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    rt.block_on(s_infra_1_api())
}

async fn s_infra_1_api() -> ScenarioOutcome {
    use crate::api_helpers;
    use axum::http::{Method, StatusCode};
    use serde_json::json;

    // ── Setup ────────────────────────────────────────────────────
    let pool = match api_helpers::scenario_pool().await {
        Ok(p) => p,
        Err(e) => return ScenarioOutcome::Blocked(e),
    };
    if let Err(e) = api_helpers::setup_db(&pool).await {
        return ScenarioOutcome::Fail(format!("DB setup failed: {e}"));
    }
    let tenant_id = match api_helpers::create_tenant(&pool, "S.INFRA.1 Round-trip").await {
        Ok(id) => id,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    let app = api_helpers::router(pool).await;

    // ── Step 1: POST /projects → 201 ────────────────────────────
    let (status, body) = match api_helpers::api_call(
        &app,
        Method::POST,
        "/projects",
        tenant_id,
        Some(json!({"client_name": "Round-Trip Test"})),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::CREATED {
        return ScenarioOutcome::Fail(format!(
            "Step 1: POST /projects: expected 201, got {status}"
        ));
    }
    let project_id = match body["id"].as_str() {
        Some(id) => id.to_string(),
        None => return ScenarioOutcome::Fail("Step 1: POST /projects: no id in response".into()),
    };

    // ── Step 2: GET /projects/:id → 200 ─────────────────────────
    let (status, body) = match api_helpers::api_call(
        &app,
        Method::GET,
        &format!("/projects/{project_id}"),
        tenant_id,
        None,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::OK {
        return ScenarioOutcome::Fail(format!(
            "Step 2: GET /projects/{project_id}: expected 200, got {status}"
        ));
    }
    if body["client_name"].as_str() != Some("Round-Trip Test") {
        return ScenarioOutcome::Fail(format!(
            "Step 2: client_name mismatch: expected \"Round-Trip Test\", got {}",
            body["client_name"]
        ));
    }

    // ── Step 3: POST /projects/:id/zones (12×15 patio) → 201 ───
    let patio_geojson = json!({
        "type": "Polygon",
        "coordinates": [[[0.0, 0.0], [12.0, 0.0], [12.0, 15.0], [0.0, 15.0], [0.0, 0.0]]]
    });
    let (status, body) = match api_helpers::api_call(
        &app,
        Method::POST,
        &format!("/projects/{project_id}/zones"),
        tenant_id,
        Some(json!({
            "geometry": patio_geojson,
            "zone_type": "patio",
            "label": "Back patio"
        })),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::CREATED {
        return ScenarioOutcome::Fail(format!(
            "Step 3: POST /projects/{project_id}/zones: expected 201, got {status}"
        ));
    }
    let zone_id = match body["id"].as_str() {
        Some(id) => id.to_string(),
        None => return ScenarioOutcome::Fail("Step 3: POST zones: no id in response".into()),
    };

    // ── Step 4: GET /projects/:id/zones → zone present ──────────
    let (status, body) = match api_helpers::api_call(
        &app,
        Method::GET,
        &format!("/projects/{project_id}/zones"),
        tenant_id,
        None,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::OK {
        return ScenarioOutcome::Fail(format!(
            "Step 4: GET /projects/{project_id}/zones: expected 200, got {status}"
        ));
    }
    let zones = match body.as_array() {
        Some(a) => a,
        None => return ScenarioOutcome::Fail("Step 4: zones response is not an array".into()),
    };
    if !zones.iter().any(|z| z["id"].as_str() == Some(&zone_id)) {
        return ScenarioOutcome::Fail(format!("Step 4: zone {zone_id} not found in zones list"));
    }

    // ── Step 5: POST /materials (Travertine Pavers, $8.50/sqft) → 201 ──
    let (status, body) = match api_helpers::api_call(
        &app,
        Method::POST,
        "/materials",
        tenant_id,
        Some(json!({
            "name": "Travertine Pavers",
            "category": "hardscape",
            "unit": "sq_ft",
            "price_per_unit": "8.50",
            "extrusion": {"type": "sits_on_top", "height_inches": 1.0}
        })),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::CREATED {
        return ScenarioOutcome::Fail(format!(
            "Step 5: POST /materials: expected 201, got {status}"
        ));
    }
    let material_id = match body["id"].as_str() {
        Some(id) => id.to_string(),
        None => return ScenarioOutcome::Fail("Step 5: POST /materials: no id in response".into()),
    };

    // ── Step 6: PUT /projects/:id/tiers/good (assign material → zone) ──
    let (status, _) = match api_helpers::api_call(
        &app,
        Method::PUT,
        &format!("/projects/{project_id}/tiers/good"),
        tenant_id,
        Some(json!({
            "assignments": [
                {"zone_id": zone_id, "material_id": material_id}
            ]
        })),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::NO_CONTENT {
        return ScenarioOutcome::Fail(format!(
            "Step 6: PUT /projects/{project_id}/tiers/good: expected 204, got {status}"
        ));
    }

    // ── Step 7: GET /projects/:id/quote/good → verify line items ──
    // Expected: 12 × 15 = 180 sqft × $8.50 = $1,530.00
    // (computed independently, not by pt-geo or pt-quote)
    let (status, quote) = match api_helpers::api_call(
        &app,
        Method::GET,
        &format!("/projects/{project_id}/quote/good"),
        tenant_id,
        None,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::OK {
        return ScenarioOutcome::Fail(format!(
            "Step 7: GET /projects/{project_id}/quote/good: expected 200, got {status}"
        ));
    }
    let line_items = match quote["line_items"].as_array() {
        Some(a) => a,
        None => return ScenarioOutcome::Fail("Step 7: line_items not an array".into()),
    };
    if line_items.len() != 1 {
        return ScenarioOutcome::Fail(format!(
            "Step 7: expected 1 line item, got {}",
            line_items.len()
        ));
    }
    // 12 × 15 = 180 sqft. 180 × $8.50 = $1,530.00
    if line_items[0]["line_total"] != "1530.00" {
        return ScenarioOutcome::Fail(format!(
            "Step 7: line_total: expected \"1530.00\", got {}",
            line_items[0]["line_total"]
        ));
    }
    if quote["subtotal"] != "1530.00" {
        return ScenarioOutcome::Fail(format!(
            "Step 7: subtotal: expected \"1530.00\", got {}",
            quote["subtotal"]
        ));
    }

    // ── Step 8: DELETE /projects/:id → 200 ──────────────────────
    let (status, _) = match api_helpers::api_call(
        &app,
        Method::DELETE,
        &format!("/projects/{project_id}"),
        tenant_id,
        None,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::OK {
        return ScenarioOutcome::Fail(format!(
            "Step 8: DELETE /projects/{project_id}: expected 200, got {status}"
        ));
    }

    // ── Step 9: GET /projects/:id → 404 ─────────────────────────
    let (status, _) = match api_helpers::api_call(
        &app,
        Method::GET,
        &format!("/projects/{project_id}"),
        tenant_id,
        None,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::NOT_FOUND {
        return ScenarioOutcome::Fail(format!(
            "Step 9: GET /projects/{project_id} after delete: expected 404, got {status}"
        ));
    }

    ScenarioOutcome::Pass(Integration::TwoStar, Polish::OneStar)
}

fn s_infra_2_tenant_isolation() -> ScenarioOutcome {
    if std::env::var("DATABASE_URL").is_err() {
        return ScenarioOutcome::Blocked(
            "no DATABASE_URL — tenant isolation requires a real database".to_string(),
        );
    }

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    rt.block_on(s_infra_2_api())
}

async fn s_infra_2_api() -> ScenarioOutcome {
    use crate::api_helpers;
    use axum::http::{Method, StatusCode};
    use serde_json::json;

    // ── Setup ────────────────────────────────────────────────────
    let pool = match api_helpers::scenario_pool().await {
        Ok(p) => p,
        Err(e) => return ScenarioOutcome::Blocked(e),
    };
    if let Err(e) = api_helpers::setup_db(&pool).await {
        return ScenarioOutcome::Fail(format!("DB setup failed: {e}"));
    }
    let tenant_a = match api_helpers::create_tenant(&pool, "S.INFRA.2 Tenant A").await {
        Ok(id) => id,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    let tenant_b = match api_helpers::create_tenant(&pool, "S.INFRA.2 Tenant B").await {
        Ok(id) => id,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    let app = api_helpers::router(pool).await;

    // ── Step 1: Tenant A creates a project → 201 ────────────────
    let (status, body) = match api_helpers::api_call(
        &app,
        Method::POST,
        "/projects",
        tenant_a,
        Some(json!({"client_name": "Tenant A's Project"})),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::CREATED {
        return ScenarioOutcome::Fail(format!(
            "POST /projects as Tenant A: expected 201, got {status}"
        ));
    }
    let project_id = match body["id"].as_str() {
        Some(id) => id.to_string(),
        None => return ScenarioOutcome::Fail("POST /projects: no id in response".to_string()),
    };

    // ── Step 2: Tenant B fetches project → 404 ──────────────────
    let (status, _) = match api_helpers::api_call(
        &app,
        Method::GET,
        &format!("/projects/{project_id}"),
        tenant_b,
        None,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::NOT_FOUND {
        return ScenarioOutcome::Fail(format!(
            "GET /projects/{project_id} as Tenant B: expected 404, got {status} (existence leak!)"
        ));
    }

    // ── Step 3: Tenant A creates a material → 201 ───────────────
    let (status, mat_body) = match api_helpers::api_call(
        &app,
        Method::POST,
        "/materials",
        tenant_a,
        Some(json!({
            "name": "Tenant A Flagstone",
            "category": "Hardscape",
            "unit": "SqFt",
            "price_per_unit": "12.50",
            "extrusion": { "SitsOnTop": { "height_inches": 2.0 } }
        })),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::CREATED {
        return ScenarioOutcome::Fail(format!(
            "POST /materials as Tenant A: expected 201, got {status}"
        ));
    }
    let material_id = match mat_body["id"].as_str() {
        Some(id) => id.to_string(),
        None => return ScenarioOutcome::Fail("POST /materials: no id in response".to_string()),
    };

    // ── Step 4: Tenant B lists materials → A's material absent ──
    let (status, materials) =
        match api_helpers::api_call(&app, Method::GET, "/materials", tenant_b, None).await {
            Ok(r) => r,
            Err(e) => return ScenarioOutcome::Fail(e),
        };
    if status != StatusCode::OK {
        return ScenarioOutcome::Fail(format!(
            "GET /materials as Tenant B: expected 200, got {status}"
        ));
    }
    if let Some(arr) = materials.as_array() {
        if arr.iter().any(|m| m["id"].as_str() == Some(&material_id)) {
            return ScenarioOutcome::Fail(
                "tenant isolation breach: Tenant B can see Tenant A's material in list".to_string(),
            );
        }
    }

    // ── Step 5: Tenant A creates a zone (setup for step 6) ──────
    let zone_geojson = json!({
        "type": "Polygon",
        "coordinates": [[[0.0, 0.0], [3.0, 0.0], [3.0, 4.0], [0.0, 4.0], [0.0, 0.0]]]
    });
    let (status, _) = match api_helpers::api_call(
        &app,
        Method::POST,
        &format!("/projects/{project_id}/zones"),
        tenant_a,
        Some(json!({
            "geometry": zone_geojson,
            "zone_type": "Patio",
            "label": "Test Patio"
        })),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::CREATED {
        return ScenarioOutcome::Fail(format!(
            "POST /projects/{project_id}/zones as Tenant A: expected 201, got {status}"
        ));
    }

    // ── Step 6: Tenant B creates zone on A's project → 404 ─────
    let (status, _) = match api_helpers::api_call(
        &app,
        Method::POST,
        &format!("/projects/{project_id}/zones"),
        tenant_b,
        Some(json!({
            "geometry": zone_geojson,
            "zone_type": "Bed",
            "label": "Intruder Zone"
        })),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::NOT_FOUND {
        return ScenarioOutcome::Fail(format!(
            "POST /projects/{project_id}/zones as Tenant B: expected 404, got {status} (isolation breach!)"
        ));
    }

    // ── Step 7: Tenant B assigns tier on A's project → 404 ─────
    let (status, _) = match api_helpers::api_call(
        &app,
        Method::PUT,
        &format!("/projects/{project_id}/tiers/good"),
        tenant_b,
        Some(json!({ "assignments": [] })),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::NOT_FOUND {
        return ScenarioOutcome::Fail(format!(
            "PUT /projects/{project_id}/tiers/good as Tenant B: expected 404, got {status} (isolation breach!)"
        ));
    }

    ScenarioOutcome::Pass(Integration::TwoStar, Polish::OneStar)
}
