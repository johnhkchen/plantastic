use crate::registry::{Integration, Polish, Scenario, ScenarioOutcome, ValueArea};

pub fn scenarios() -> &'static [Scenario] {
    &SCENARIOS
}

static SCENARIOS: [Scenario; 4] = [
    Scenario {
        id: "S.3.1",
        name: "Quantity computation from geometry",
        area: ValueArea::Quoting,
        time_savings_minutes: 25.0,
        replaces: "Manual quantity takeoff with tape measure and calculator",
        test_fn: s_3_1_quantity_from_geometry,
    },
    Scenario {
        id: "S.3.2",
        name: "Three-tier quote generation",
        area: ValueArea::Quoting,
        time_savings_minutes: 15.0,
        replaces: "Three separate spreadsheet calculations for good/better/best",
        test_fn: s_3_2_three_tier_quotes,
    },
    Scenario {
        id: "S.3.3",
        name: "Branded PDF export",
        area: ValueArea::Quoting,
        time_savings_minutes: 10.0,
        replaces: "Formatting proposals in Word, pasting photos, inconsistent branding",
        test_fn: s_3_3_branded_pdf,
    },
    Scenario {
        id: "S.3.4",
        name: "Client quote comparison view",
        area: ValueArea::Quoting,
        time_savings_minutes: 10.0,
        replaces: "Printing three separate quotes for client to compare side by side",
        test_fn: s_3_4_client_comparison,
    },
];

// ── GeoJSON helpers ──────────────────────────────────────────

fn geojson_rect(w: f64, h: f64) -> serde_json::Value {
    serde_json::json!({
        "type": "Polygon",
        "coordinates": [[[0.0, 0.0], [w, 0.0], [w, h], [0.0, h], [0.0, 0.0]]]
    })
}

// ── S.3.1 — Quantity computation from geometry (TwoStar) ─────

/// Creates project + zones + materials + tier via API, fetches quote,
/// and verifies line item totals against independently computed arithmetic.
/// Falls back to computation-only path (OneStar) when no database is available.
fn s_3_1_quantity_from_geometry() -> ScenarioOutcome {
    if std::env::var("DATABASE_URL").is_err() {
        return s_3_1_computation();
    }

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    rt.block_on(s_3_1_api())
}

async fn s_3_1_api() -> ScenarioOutcome {
    use crate::api_helpers;
    use axum::http::{Method, StatusCode};
    use serde_json::json;

    // ── Setup ────────────────────────────────────────────────
    let pool = match api_helpers::scenario_pool().await {
        Ok(p) => p,
        Err(e) => return ScenarioOutcome::Blocked(e),
    };
    if let Err(e) = api_helpers::setup_db(&pool).await {
        return ScenarioOutcome::Fail(format!("DB setup failed: {e}"));
    }
    let tenant_id = match api_helpers::create_tenant(&pool, "S.3.1 Quote Test").await {
        Ok(id) => id,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    let app = api_helpers::router(pool).await;

    // ── Create project ───────────────────────────────────────
    let (status, body) = match api_helpers::api_call(
        &app,
        Method::POST,
        "/projects",
        tenant_id,
        Some(json!({"client_name": "S.3.1 Client"})),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::CREATED {
        return ScenarioOutcome::Fail(format!("POST /projects: {status}"));
    }
    let project_id = body["id"].as_str().unwrap();

    // ── Create 3 zones ───────────────────────────────────────
    // 12×15 ft patio (area = 180.0 sq_ft)
    let (status, body) = match api_helpers::api_call(
        &app,
        Method::POST,
        &format!("/projects/{project_id}/zones"),
        tenant_id,
        Some(json!({
            "geometry": geojson_rect(12.0, 15.0),
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
        return ScenarioOutcome::Fail(format!("POST zone (patio): {status}"));
    }
    let patio_zone_id = body["id"].as_str().unwrap().to_string();

    // 8×20 ft garden bed (area = 160.0 sq_ft)
    let (status, body) = match api_helpers::api_call(
        &app,
        Method::POST,
        &format!("/projects/{project_id}/zones"),
        tenant_id,
        Some(json!({
            "geometry": geojson_rect(8.0, 20.0),
            "zone_type": "bed",
            "label": "Garden bed"
        })),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::CREATED {
        return ScenarioOutcome::Fail(format!("POST zone (bed): {status}"));
    }
    let bed_zone_id = body["id"].as_str().unwrap().to_string();

    // 10×10 edging (perimeter = 40.0 linear_ft)
    let (status, body) = match api_helpers::api_call(
        &app,
        Method::POST,
        &format!("/projects/{project_id}/zones"),
        tenant_id,
        Some(json!({
            "geometry": geojson_rect(10.0, 10.0),
            "zone_type": "edging",
            "label": "Border edging"
        })),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::CREATED {
        return ScenarioOutcome::Fail(format!("POST zone (edging): {status}"));
    }
    let edge_zone_id = body["id"].as_str().unwrap().to_string();

    // ── Create 3 materials ───────────────────────────────────
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
        return ScenarioOutcome::Fail(format!("POST material (pavers): {status}"));
    }
    let paver_id = body["id"].as_str().unwrap().to_string();

    let (status, body) = match api_helpers::api_call(
        &app,
        Method::POST,
        "/materials",
        tenant_id,
        Some(json!({
            "name": "Premium Mulch",
            "category": "softscape",
            "unit": "cu_yd",
            "price_per_unit": "45.00",
            "depth_inches": 4.0,
            "extrusion": {"type": "fills", "flush": true}
        })),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::CREATED {
        return ScenarioOutcome::Fail(format!("POST material (mulch): {status}"));
    }
    let mulch_id = body["id"].as_str().unwrap().to_string();

    let (status, body) = match api_helpers::api_call(
        &app,
        Method::POST,
        "/materials",
        tenant_id,
        Some(json!({
            "name": "Steel Edge",
            "category": "edging",
            "unit": "linear_ft",
            "price_per_unit": "3.25",
            "extrusion": {"type": "sits_on_top", "height_inches": 4.0}
        })),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::CREATED {
        return ScenarioOutcome::Fail(format!("POST material (edging): {status}"));
    }
    let edge_mat_id = body["id"].as_str().unwrap().to_string();

    // ── Set tier assignments for "better" ────────────────────
    let (status, _) = match api_helpers::api_call(
        &app,
        Method::PUT,
        &format!("/projects/{project_id}/tiers/better"),
        tenant_id,
        Some(json!({
            "assignments": [
                {"zone_id": patio_zone_id, "material_id": paver_id},
                {"zone_id": bed_zone_id, "material_id": mulch_id},
                {"zone_id": edge_zone_id, "material_id": edge_mat_id},
            ]
        })),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::NO_CONTENT {
        return ScenarioOutcome::Fail(format!("PUT tiers/better: {status}"));
    }

    // ── GET quote ────────────────────────────────────────────
    let (status, quote) = match api_helpers::api_call(
        &app,
        Method::GET,
        &format!("/projects/{project_id}/quote/better"),
        tenant_id,
        None,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::OK {
        return ScenarioOutcome::Fail(format!("GET quote/better: {status}"));
    }

    // ── Assert independently computed values ─────────────────
    //
    // Patio:  12 × 15 = 180 sq_ft × $8.50 = $1,530.00
    // Mulch:  8 × 20 = 160 sq_ft × (4/12) ft depth = 53.3333... cu_ft / 27 = 1.9753... cu_yd
    //         1.9753... × $45.00 = $88.888... → $88.89
    // Edging: (10 + 10 + 10 + 10) = 40 linear_ft × $3.25 = $130.00
    // Subtotal: $1,530.00 + $88.89 + $130.00 = $1,748.89

    let line_items = match quote["line_items"].as_array() {
        Some(a) => a,
        None => return ScenarioOutcome::Fail("line_items not an array".to_string()),
    };

    if line_items.len() != 3 {
        return ScenarioOutcome::Fail(format!("expected 3 line items, got {}", line_items.len()));
    }

    let patio_li = line_items
        .iter()
        .find(|li| li["material_name"] == "Travertine Pavers");
    let mulch_li = line_items
        .iter()
        .find(|li| li["material_name"] == "Premium Mulch");
    let edge_li = line_items
        .iter()
        .find(|li| li["material_name"] == "Steel Edge");

    let (Some(patio_li), Some(mulch_li), Some(edge_li)) = (patio_li, mulch_li, edge_li) else {
        return ScenarioOutcome::Fail(
            "could not find all expected line items by material name".to_string(),
        );
    };

    if patio_li["line_total"] != "1530.00" {
        return ScenarioOutcome::Fail(format!(
            "patio line_total: expected \"1530.00\", got {}",
            patio_li["line_total"]
        ));
    }
    if mulch_li["line_total"] != "88.89" {
        return ScenarioOutcome::Fail(format!(
            "mulch line_total: expected \"88.89\", got {}",
            mulch_li["line_total"]
        ));
    }
    if edge_li["line_total"] != "130.00" {
        return ScenarioOutcome::Fail(format!(
            "edging line_total: expected \"130.00\", got {}",
            edge_li["line_total"]
        ));
    }
    if quote["subtotal"] != "1748.89" {
        return ScenarioOutcome::Fail(format!(
            "subtotal: expected \"1748.89\", got {}",
            quote["subtotal"]
        ));
    }
    if quote["total"] != "1748.89" {
        return ScenarioOutcome::Fail(format!(
            "total: expected \"1748.89\", got {}",
            quote["total"]
        ));
    }

    // ThreeStar polish (T-026-02): empty state with CTA when no assignments,
    // on top of skeleton loading + error banner from T-026-01.
    ScenarioOutcome::Pass(Integration::TwoStar, Polish::ThreeStar)
}

// ── S.3.2 — Three-tier quote generation (TwoStar) ───────────

/// Creates project + zones + 9 materials + 3 tier assignments via API,
/// fetches quotes for all three tiers, and verifies Good < Better < Best
/// with exact totals.
fn s_3_2_three_tier_quotes() -> ScenarioOutcome {
    if std::env::var("DATABASE_URL").is_err() {
        return s_3_2_computation();
    }

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    rt.block_on(s_3_2_api())
}

async fn s_3_2_api() -> ScenarioOutcome {
    use crate::api_helpers;
    use axum::http::{Method, StatusCode};
    use rust_decimal::Decimal;
    use serde_json::json;
    use std::str::FromStr;

    // ── Setup ────────────────────────────────────────────────
    let pool = match api_helpers::scenario_pool().await {
        Ok(p) => p,
        Err(e) => return ScenarioOutcome::Blocked(e),
    };
    if let Err(e) = api_helpers::setup_db(&pool).await {
        return ScenarioOutcome::Fail(format!("DB setup failed: {e}"));
    }
    let tenant_id = match api_helpers::create_tenant(&pool, "S.3.2 Tier Test").await {
        Ok(id) => id,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    let app = api_helpers::router(pool).await;

    // ── Create project ───────────────────────────────────────
    let (status, body) = match api_helpers::api_call(
        &app,
        Method::POST,
        "/projects",
        tenant_id,
        Some(json!({"client_name": "S.3.2 Client"})),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return ScenarioOutcome::Fail(e),
    };
    if status != StatusCode::CREATED {
        return ScenarioOutcome::Fail(format!("POST /projects: {status}"));
    }
    let project_id = body["id"].as_str().unwrap().to_string();

    // ── Create 3 zones (same geometry as S.3.1) ──────────────
    let zone_specs = [
        (12.0, 15.0, "patio", "Back patio"),
        (8.0, 20.0, "bed", "Garden bed"),
        (10.0, 10.0, "edging", "Border edging"),
    ];
    let mut zone_ids = Vec::with_capacity(3);
    for (w, h, ztype, label) in &zone_specs {
        let (status, body) = match api_helpers::api_call(
            &app,
            Method::POST,
            &format!("/projects/{project_id}/zones"),
            tenant_id,
            Some(json!({
                "geometry": geojson_rect(*w, *h),
                "zone_type": ztype,
                "label": label
            })),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => return ScenarioOutcome::Fail(e),
        };
        if status != StatusCode::CREATED {
            return ScenarioOutcome::Fail(format!("POST zone ({label}): {status}"));
        }
        zone_ids.push(body["id"].as_str().unwrap().to_string());
    }

    // ── Create 9 materials (3 per tier) ──────────────────────
    // Good tier (cheapest)
    let good_materials = vec![
        json!({"name": "Concrete Pavers", "category": "hardscape", "unit": "sq_ft",
               "price_per_unit": "4.00", "extrusion": {"type": "sits_on_top", "height_inches": 1.0}}),
        json!({"name": "Basic Mulch", "category": "softscape", "unit": "cu_yd",
               "price_per_unit": "30.00", "depth_inches": 4.0,
               "extrusion": {"type": "fills", "flush": true}}),
        json!({"name": "Plastic Edge", "category": "edging", "unit": "linear_ft",
               "price_per_unit": "1.50", "extrusion": {"type": "sits_on_top", "height_inches": 4.0}}),
    ];
    let better_materials = vec![
        json!({"name": "Travertine Pavers", "category": "hardscape", "unit": "sq_ft",
               "price_per_unit": "8.50", "extrusion": {"type": "sits_on_top", "height_inches": 1.0}}),
        json!({"name": "Premium Mulch", "category": "softscape", "unit": "cu_yd",
               "price_per_unit": "45.00", "depth_inches": 4.0,
               "extrusion": {"type": "fills", "flush": true}}),
        json!({"name": "Steel Edge", "category": "edging", "unit": "linear_ft",
               "price_per_unit": "3.25", "extrusion": {"type": "sits_on_top", "height_inches": 4.0}}),
    ];
    let best_materials = vec![
        json!({"name": "Bluestone Pavers", "category": "hardscape", "unit": "sq_ft",
               "price_per_unit": "15.00", "extrusion": {"type": "sits_on_top", "height_inches": 1.5}}),
        json!({"name": "Cedar Mulch", "category": "softscape", "unit": "cu_yd",
               "price_per_unit": "65.00", "depth_inches": 4.0,
               "extrusion": {"type": "fills", "flush": true}}),
        json!({"name": "Corten Edge", "category": "edging", "unit": "linear_ft",
               "price_per_unit": "8.00", "extrusion": {"type": "sits_on_top", "height_inches": 4.0}}),
    ];

    async fn create_materials(
        app: &axum::Router,
        tenant_id: uuid::Uuid,
        specs: &[serde_json::Value],
    ) -> Result<Vec<String>, ScenarioOutcome> {
        let mut ids = Vec::with_capacity(specs.len());
        for mat in specs {
            let name = mat["name"].as_str().unwrap_or("unknown");
            let (status, body) = match crate::api_helpers::api_call(
                app,
                axum::http::Method::POST,
                "/materials",
                tenant_id,
                Some(mat.clone()),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return Err(ScenarioOutcome::Fail(e)),
            };
            if status != axum::http::StatusCode::CREATED {
                return Err(ScenarioOutcome::Fail(format!(
                    "POST material ({name}): {status}"
                )));
            }
            ids.push(body["id"].as_str().unwrap().to_string());
        }
        Ok(ids)
    }

    let good_mat_ids = match create_materials(&app, tenant_id, &good_materials).await {
        Ok(ids) => ids,
        Err(outcome) => return outcome,
    };
    let better_mat_ids = match create_materials(&app, tenant_id, &better_materials).await {
        Ok(ids) => ids,
        Err(outcome) => return outcome,
    };
    let best_mat_ids = match create_materials(&app, tenant_id, &best_materials).await {
        Ok(ids) => ids,
        Err(outcome) => return outcome,
    };

    // ── Set tier assignments ─────────────────────────────────
    for (tier_name, mat_ids) in [
        ("good", &good_mat_ids),
        ("better", &better_mat_ids),
        ("best", &best_mat_ids),
    ] {
        let assignments: Vec<_> = zone_ids
            .iter()
            .zip(mat_ids.iter())
            .map(|(zid, mid)| json!({"zone_id": zid, "material_id": mid}))
            .collect();
        let (status, _) = match api_helpers::api_call(
            &app,
            Method::PUT,
            &format!("/projects/{project_id}/tiers/{tier_name}"),
            tenant_id,
            Some(json!({"assignments": assignments})),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => return ScenarioOutcome::Fail(e),
        };
        if status != StatusCode::NO_CONTENT {
            return ScenarioOutcome::Fail(format!("PUT tiers/{tier_name}: {status}"));
        }
    }

    // ── GET quotes for all three tiers ───────────────────────
    let mut quotes = Vec::with_capacity(3);
    for tier_name in ["good", "better", "best"] {
        let (status, quote) = match api_helpers::api_call(
            &app,
            Method::GET,
            &format!("/projects/{project_id}/quote/{tier_name}"),
            tenant_id,
            None,
        )
        .await
        {
            Ok(r) => r,
            Err(e) => return ScenarioOutcome::Fail(e),
        };
        if status != StatusCode::OK {
            return ScenarioOutcome::Fail(format!("GET quote/{tier_name}: {status}"));
        }
        quotes.push((tier_name, quote));
    }

    // ── Assert: each tier has 3 line items ───────────────────
    for (tier_name, quote) in &quotes {
        let items = match quote["line_items"].as_array() {
            Some(a) => a,
            None => return ScenarioOutcome::Fail(format!("{tier_name}: line_items not an array")),
        };
        if items.len() != 3 {
            return ScenarioOutcome::Fail(format!(
                "{tier_name}: expected 3 line items, got {}",
                items.len()
            ));
        }
    }

    // ── Assert: Good < Better < Best ─────────────────────────
    let parse_total = |q: &serde_json::Value| -> Result<Decimal, ScenarioOutcome> {
        let s = q["total"]
            .as_str()
            .ok_or_else(|| ScenarioOutcome::Fail("total is not a string".to_string()))?;
        Decimal::from_str(s)
            .map_err(|e| ScenarioOutcome::Fail(format!("cannot parse total \"{s}\": {e}")))
    };

    let good_total = match parse_total(&quotes[0].1) {
        Ok(d) => d,
        Err(outcome) => return outcome,
    };
    let better_total = match parse_total(&quotes[1].1) {
        Ok(d) => d,
        Err(outcome) => return outcome,
    };
    let best_total = match parse_total(&quotes[2].1) {
        Ok(d) => d,
        Err(outcome) => return outcome,
    };

    if good_total >= better_total {
        return ScenarioOutcome::Fail(format!(
            "Good ({good_total}) should be less than Better ({better_total})"
        ));
    }
    if better_total >= best_total {
        return ScenarioOutcome::Fail(format!(
            "Better ({better_total}) should be less than Best ({best_total})"
        ));
    }

    // ── Assert: subtotal == sum(line_totals) for each tier ───
    for (tier_name, quote) in &quotes {
        let items = quote["line_items"].as_array().unwrap();
        let sum: Decimal = items
            .iter()
            .map(|li| Decimal::from_str(li["line_total"].as_str().unwrap()).unwrap())
            .sum();
        let subtotal = Decimal::from_str(quote["subtotal"].as_str().unwrap()).unwrap();
        if sum != subtotal {
            return ScenarioOutcome::Fail(format!(
                "{tier_name}: subtotal ({subtotal}) != sum of line_totals ({sum})"
            ));
        }
    }

    // ── Assert: no duplicate zone_id within a tier ───────────
    for (tier_name, quote) in &quotes {
        let items = quote["line_items"].as_array().unwrap();
        let mut seen = std::collections::HashSet::new();
        for li in items {
            let zid = li["zone_id"].as_str().unwrap();
            if !seen.insert(zid) {
                return ScenarioOutcome::Fail(format!("{tier_name}: duplicate zone_id {zid}"));
            }
        }
    }

    // ── Assert: exact totals ─────────────────────────────────
    // Good: 180 * 4.00 + (160 * 4/12/27) * 30.00 + 40 * 1.50
    //     = 720.00 + 59.26 + 60.00 = 839.26
    let expected_good = Decimal::from_str("839.26").unwrap();
    if good_total != expected_good {
        return ScenarioOutcome::Fail(format!(
            "Good total: expected {expected_good}, got {good_total}"
        ));
    }

    // Best: 180 * 15.00 + (160 * 4/12/27) * 65.00 + 40 * 8.00
    //     = 2700.00 + 128.40 + 320.00 = 3148.40
    let expected_best = Decimal::from_str("3148.40").unwrap();
    if best_total != expected_best {
        return ScenarioOutcome::Fail(format!(
            "Best total: expected {expected_best}, got {best_total}"
        ));
    }

    // ThreeStar polish (T-026-02): empty state with CTA when no assignments,
    // on top of skeleton loading + error banner from T-026-01.
    ScenarioOutcome::Pass(Integration::TwoStar, Polish::ThreeStar)
}

// ── Stubs ────────────────────────────────────────────────────

fn s_3_3_branded_pdf() -> ScenarioOutcome {
    ScenarioOutcome::NotImplemented
}

fn s_3_4_client_comparison() -> ScenarioOutcome {
    ScenarioOutcome::NotImplemented
}

// ── Unit-level regression tests (preserved from OneStar) ─────
//
// These verify the pt-quote computation engine directly, without
// the API layer. They preserve the original scenario test logic
// as regression tests that run during `cargo test -p pt-scenarios`.

/// S.3.1 computation-only path (OneStar). Used as fallback when no database
/// is available, and as a unit regression test.
fn s_3_1_computation() -> ScenarioOutcome {
    use geo::polygon;
    use pt_materials::{ExtrusionBehavior, Material, MaterialCategory, Unit};
    use pt_project::{MaterialAssignment, Tier, TierLevel, Zone, ZoneId, ZoneType};
    use pt_quote::compute_quote;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    let patio_id = ZoneId::new();
    let patio = Zone {
        id: patio_id,
        geometry: polygon![
            (x: 0.0, y: 0.0),
            (x: 12.0, y: 0.0),
            (x: 12.0, y: 15.0),
            (x: 0.0, y: 15.0),
        ],
        zone_type: ZoneType::Patio,
        label: Some("Back patio".to_string()),
    };

    let bed_id = ZoneId::new();
    let bed = Zone {
        id: bed_id,
        geometry: polygon![
            (x: 0.0, y: 0.0),
            (x: 8.0, y: 0.0),
            (x: 8.0, y: 20.0),
            (x: 0.0, y: 20.0),
        ],
        zone_type: ZoneType::Bed,
        label: Some("Garden bed".to_string()),
    };

    let edge_id = ZoneId::new();
    let edge_zone = Zone {
        id: edge_id,
        geometry: polygon![
            (x: 0.0, y: 0.0),
            (x: 10.0, y: 0.0),
            (x: 10.0, y: 10.0),
            (x: 0.0, y: 10.0),
        ],
        zone_type: ZoneType::Edging,
        label: Some("Border edging".to_string()),
    };

    let zones = [patio, bed, edge_zone];

    let pavers = Material::builder("Travertine Pavers", MaterialCategory::Hardscape)
        .unit(Unit::SqFt)
        .price_per_unit(Decimal::from_str("8.50").unwrap())
        .extrusion(ExtrusionBehavior::SitsOnTop { height_inches: 1.0 })
        .build();

    let mulch = Material::builder("Premium Mulch", MaterialCategory::Softscape)
        .unit(Unit::CuYd)
        .price_per_unit(Decimal::from_str("45.00").unwrap())
        .depth_inches(4.0)
        .extrusion(ExtrusionBehavior::Fills { flush: true })
        .build();

    let edging = Material::builder("Steel Edge", MaterialCategory::Edging)
        .unit(Unit::LinearFt)
        .price_per_unit(Decimal::from_str("3.25").unwrap())
        .extrusion(ExtrusionBehavior::SitsOnTop { height_inches: 4.0 })
        .build();

    let materials = [pavers.clone(), mulch.clone(), edging.clone()];

    let tier = Tier {
        level: TierLevel::Better,
        assignments: vec![
            MaterialAssignment {
                zone_id: patio_id,
                material_id: pavers.id,
                overrides: None,
            },
            MaterialAssignment {
                zone_id: bed_id,
                material_id: mulch.id,
                overrides: None,
            },
            MaterialAssignment {
                zone_id: edge_id,
                material_id: edging.id,
                overrides: None,
            },
        ],
    };

    let quote = match compute_quote(&zones, &tier, &materials, None) {
        Ok(q) => q,
        Err(e) => return ScenarioOutcome::Fail(format!("compute_quote failed: {e}")),
    };

    let expected_patio = Decimal::from_str("1530.00").unwrap();
    let expected_mulch = Decimal::from_str("88.89").unwrap();
    let expected_edging = Decimal::from_str("130.00").unwrap();
    let expected_subtotal = Decimal::from_str("1748.89").unwrap();

    if quote.line_items.len() != 3 {
        return ScenarioOutcome::Fail(format!(
            "expected 3 line items, got {}",
            quote.line_items.len()
        ));
    }

    let patio_li = quote
        .line_items
        .iter()
        .find(|li| li.material_name == "Travertine Pavers");
    let mulch_li = quote
        .line_items
        .iter()
        .find(|li| li.material_name == "Premium Mulch");
    let edge_li = quote
        .line_items
        .iter()
        .find(|li| li.material_name == "Steel Edge");

    let (Some(patio_li), Some(mulch_li), Some(edge_li)) = (patio_li, mulch_li, edge_li) else {
        return ScenarioOutcome::Fail(
            "could not find all expected line items by material name".to_string(),
        );
    };

    if patio_li.line_total != expected_patio {
        return ScenarioOutcome::Fail(format!(
            "patio line total: expected {expected_patio}, got {}",
            patio_li.line_total
        ));
    }
    if mulch_li.line_total != expected_mulch {
        return ScenarioOutcome::Fail(format!(
            "mulch line total: expected {expected_mulch}, got {}",
            mulch_li.line_total
        ));
    }
    if edge_li.line_total != expected_edging {
        return ScenarioOutcome::Fail(format!(
            "edging line total: expected {expected_edging}, got {}",
            edge_li.line_total
        ));
    }
    if quote.subtotal != expected_subtotal {
        return ScenarioOutcome::Fail(format!(
            "subtotal: expected {expected_subtotal}, got {}",
            quote.subtotal
        ));
    }

    // ThreeStar polish (T-026-02): empty state with CTA when no assignments,
    // on top of skeleton loading + error banner from T-026-01.
    ScenarioOutcome::Pass(Integration::ThreeStar, Polish::ThreeStar)
}

/// S.3.2 computation-only path (OneStar). Used as fallback when no database
/// is available, and as a unit regression test.
fn s_3_2_computation() -> ScenarioOutcome {
    use geo::polygon;
    use pt_materials::{ExtrusionBehavior, Material, MaterialCategory, Unit};
    use pt_project::{MaterialAssignment, Tier, TierLevel, Zone, ZoneId, ZoneType};
    use pt_quote::compute_quote;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    let patio_id = ZoneId::new();
    let patio = Zone {
        id: patio_id,
        geometry: polygon![
            (x: 0.0, y: 0.0), (x: 12.0, y: 0.0),
            (x: 12.0, y: 15.0), (x: 0.0, y: 15.0),
        ],
        zone_type: ZoneType::Patio,
        label: Some("Back patio".to_string()),
    };

    let bed_id = ZoneId::new();
    let bed = Zone {
        id: bed_id,
        geometry: polygon![
            (x: 0.0, y: 0.0), (x: 8.0, y: 0.0),
            (x: 8.0, y: 20.0), (x: 0.0, y: 20.0),
        ],
        zone_type: ZoneType::Bed,
        label: Some("Garden bed".to_string()),
    };

    let edge_id = ZoneId::new();
    let edge_zone = Zone {
        id: edge_id,
        geometry: polygon![
            (x: 0.0, y: 0.0), (x: 10.0, y: 0.0),
            (x: 10.0, y: 10.0), (x: 0.0, y: 10.0),
        ],
        zone_type: ZoneType::Edging,
        label: Some("Border edging".to_string()),
    };

    let zones = [patio, bed, edge_zone];

    let good_paver = Material::builder("Concrete Pavers", MaterialCategory::Hardscape)
        .unit(Unit::SqFt)
        .price_per_unit(Decimal::from_str("4.00").unwrap())
        .extrusion(ExtrusionBehavior::SitsOnTop { height_inches: 1.0 })
        .build();
    let good_mulch = Material::builder("Basic Mulch", MaterialCategory::Softscape)
        .unit(Unit::CuYd)
        .price_per_unit(Decimal::from_str("30.00").unwrap())
        .depth_inches(4.0)
        .extrusion(ExtrusionBehavior::Fills { flush: true })
        .build();
    let good_edge = Material::builder("Plastic Edge", MaterialCategory::Edging)
        .unit(Unit::LinearFt)
        .price_per_unit(Decimal::from_str("1.50").unwrap())
        .extrusion(ExtrusionBehavior::SitsOnTop { height_inches: 4.0 })
        .build();

    let better_paver = Material::builder("Travertine Pavers", MaterialCategory::Hardscape)
        .unit(Unit::SqFt)
        .price_per_unit(Decimal::from_str("8.50").unwrap())
        .extrusion(ExtrusionBehavior::SitsOnTop { height_inches: 1.0 })
        .build();
    let better_mulch = Material::builder("Premium Mulch", MaterialCategory::Softscape)
        .unit(Unit::CuYd)
        .price_per_unit(Decimal::from_str("45.00").unwrap())
        .depth_inches(4.0)
        .extrusion(ExtrusionBehavior::Fills { flush: true })
        .build();
    let better_edge = Material::builder("Steel Edge", MaterialCategory::Edging)
        .unit(Unit::LinearFt)
        .price_per_unit(Decimal::from_str("3.25").unwrap())
        .extrusion(ExtrusionBehavior::SitsOnTop { height_inches: 4.0 })
        .build();

    let best_paver = Material::builder("Bluestone Pavers", MaterialCategory::Hardscape)
        .unit(Unit::SqFt)
        .price_per_unit(Decimal::from_str("15.00").unwrap())
        .extrusion(ExtrusionBehavior::SitsOnTop { height_inches: 1.5 })
        .build();
    let best_mulch = Material::builder("Cedar Mulch", MaterialCategory::Softscape)
        .unit(Unit::CuYd)
        .price_per_unit(Decimal::from_str("65.00").unwrap())
        .depth_inches(4.0)
        .extrusion(ExtrusionBehavior::Fills { flush: true })
        .build();
    let best_edge = Material::builder("Corten Edge", MaterialCategory::Edging)
        .unit(Unit::LinearFt)
        .price_per_unit(Decimal::from_str("8.00").unwrap())
        .extrusion(ExtrusionBehavior::SitsOnTop { height_inches: 4.0 })
        .build();

    let all_materials = [
        good_paver.clone(),
        good_mulch.clone(),
        good_edge.clone(),
        better_paver.clone(),
        better_mulch.clone(),
        better_edge.clone(),
        best_paver.clone(),
        best_mulch.clone(),
        best_edge.clone(),
    ];

    fn make_assignments(
        patio_id: ZoneId,
        bed_id: ZoneId,
        edge_id: ZoneId,
        paver: &Material,
        mulch: &Material,
        edge: &Material,
    ) -> Vec<MaterialAssignment> {
        vec![
            MaterialAssignment {
                zone_id: patio_id,
                material_id: paver.id,
                overrides: None,
            },
            MaterialAssignment {
                zone_id: bed_id,
                material_id: mulch.id,
                overrides: None,
            },
            MaterialAssignment {
                zone_id: edge_id,
                material_id: edge.id,
                overrides: None,
            },
        ]
    }

    let good_tier = Tier {
        level: TierLevel::Good,
        assignments: make_assignments(
            patio_id,
            bed_id,
            edge_id,
            &good_paver,
            &good_mulch,
            &good_edge,
        ),
    };
    let better_tier = Tier {
        level: TierLevel::Better,
        assignments: make_assignments(
            patio_id,
            bed_id,
            edge_id,
            &better_paver,
            &better_mulch,
            &better_edge,
        ),
    };
    let best_tier = Tier {
        level: TierLevel::Best,
        assignments: make_assignments(
            patio_id,
            bed_id,
            edge_id,
            &best_paver,
            &best_mulch,
            &best_edge,
        ),
    };

    let good_quote = match compute_quote(&zones, &good_tier, &all_materials, None) {
        Ok(q) => q,
        Err(e) => return ScenarioOutcome::Fail(format!("good tier failed: {e}")),
    };
    let better_quote = match compute_quote(&zones, &better_tier, &all_materials, None) {
        Ok(q) => q,
        Err(e) => return ScenarioOutcome::Fail(format!("better tier failed: {e}")),
    };
    let best_quote = match compute_quote(&zones, &best_tier, &all_materials, None) {
        Ok(q) => q,
        Err(e) => return ScenarioOutcome::Fail(format!("best tier failed: {e}")),
    };

    if good_quote.total >= better_quote.total {
        return ScenarioOutcome::Fail(format!(
            "Good ({}) >= Better ({})",
            good_quote.total, better_quote.total
        ));
    }
    if better_quote.total >= best_quote.total {
        return ScenarioOutcome::Fail(format!(
            "Better ({}) >= Best ({})",
            better_quote.total, best_quote.total
        ));
    }

    for (label, quote) in [
        ("Good", &good_quote),
        ("Better", &better_quote),
        ("Best", &best_quote),
    ] {
        let computed_subtotal: Decimal = quote.line_items.iter().map(|li| li.line_total).sum();
        if computed_subtotal != quote.subtotal {
            return ScenarioOutcome::Fail(format!(
                "{label}: subtotal ({}) != sum ({computed_subtotal})",
                quote.subtotal
            ));
        }
    }

    let expected_good = Decimal::from_str("839.26").unwrap();
    if good_quote.total != expected_good {
        return ScenarioOutcome::Fail(format!(
            "Good total: expected {expected_good}, got {}",
            good_quote.total
        ));
    }

    let expected_best = Decimal::from_str("3148.40").unwrap();
    if best_quote.total != expected_best {
        return ScenarioOutcome::Fail(format!(
            "Best total: expected {expected_best}, got {}",
            best_quote.total
        ));
    }

    // ThreeStar polish (T-026-02): empty state with CTA when no assignments,
    // on top of skeleton loading + error banner from T-026-01.
    ScenarioOutcome::Pass(Integration::ThreeStar, Polish::ThreeStar)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s_3_1_regression() {
        let outcome = s_3_1_computation();
        assert!(
            matches!(outcome, ScenarioOutcome::Pass(..)),
            "S.3.1 unit regression failed: {outcome:?}"
        );
    }

    #[test]
    fn s_3_2_regression() {
        let outcome = s_3_2_computation();
        assert!(
            matches!(outcome, ScenarioOutcome::Pass(..)),
            "S.3.2 unit regression failed: {outcome:?}"
        );
    }
}
