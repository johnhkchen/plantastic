//! Proposal PDF generation route.

use axum::extract::{Path, State};
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use pt_project::TierLevel;
use uuid::Uuid;

use crate::error::AppError;
use crate::extract::TenantId;
use crate::routes::shared::{build_tier, material_rows_to_materials, zone_rows_to_zones};
use crate::routes::zones::verify_project_tenant;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/projects/{id}/proposal", get(get_proposal))
}

// ── Handler ──────────────────────────────────────────────────

async fn get_proposal(
    tenant: TenantId,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    verify_project_tenant(&state.pool, project_id, tenant.0).await?;

    // Load project and tenant for branding.
    let project_row = pt_repo::project::get_by_id(&state.pool, project_id).await?;
    let tenant_row = pt_repo::tenant::get_by_id(&state.pool, project_row.tenant_id).await?;

    // Load zones and materials.
    let zone_rows = pt_repo::zone::list_by_project(&state.pool, project_id).await?;
    let material_rows =
        pt_repo::material::list_by_tenant(&state.pool, project_row.tenant_id).await?;

    let zones = zone_rows_to_zones(zone_rows);
    let materials = material_rows_to_materials(material_rows);

    // Load all 3 tier assignments.
    let good_assignments =
        pt_repo::tier_assignment::get_by_project_and_tier(&state.pool, project_id, TierLevel::Good)
            .await?;
    let better_assignments = pt_repo::tier_assignment::get_by_project_and_tier(
        &state.pool,
        project_id,
        TierLevel::Better,
    )
    .await?;
    let best_assignments =
        pt_repo::tier_assignment::get_by_project_and_tier(&state.pool, project_id, TierLevel::Best)
            .await?;

    // At least one tier must have assignments.
    if good_assignments.is_empty() && better_assignments.is_empty() && best_assignments.is_empty() {
        return Err(AppError::BadRequest(
            "project has no material assignments".to_string(),
        ));
    }

    let good_tier = build_tier(TierLevel::Good, good_assignments)?;
    let better_tier = build_tier(TierLevel::Better, better_assignments)?;
    let best_tier = build_tier(TierLevel::Best, best_assignments)?;

    // Compute 3 quotes.
    let good_quote = pt_quote::compute_quote(&zones, &good_tier, &materials, None)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    let better_quote = pt_quote::compute_quote(&zones, &better_tier, &materials, None)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    let best_quote = pt_quote::compute_quote(&zones, &best_tier, &materials, None)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    // Build ProposalInput for narrative generation.
    let proposal_input = build_proposal_input(
        &tenant_row.name,
        &project_row,
        &zones,
        &materials,
        &good_quote,
        &better_quote,
        &best_quote,
    );

    // Generate narrative (async — uses mock in tests, BAML in prod).
    let narrative = state.proposal_generator.generate(&proposal_input).await?;

    // Extract branding from tenant.
    let (phone, email) = extract_contact(&tenant_row.contact);
    let branding = pt_proposal::TenantBranding {
        company_name: tenant_row.name.clone(),
        logo_url: tenant_row.logo_url.clone(),
        primary_color: tenant_row.brand_color.clone(),
        phone,
        email,
    };

    let doc = pt_proposal::ProposalDocument {
        project_name: project_row
            .client_name
            .clone()
            .unwrap_or_else(|| "Untitled Project".to_string()),
        project_address: project_row
            .address
            .clone()
            .unwrap_or_else(|| "Address TBD".to_string()),
        date: chrono::Utc::now().format("%B %-d, %Y").to_string(),
        branding,
        narrative,
        good_quote,
        better_quote,
        best_quote,
    };

    // PDF rendering is CPU-bound — use spawn_blocking.
    let pdf_bytes = tokio::task::spawn_blocking(move || pt_proposal::render_proposal(&doc))
        .await
        .map_err(|e| AppError::Internal(format!("PDF render task panicked: {e}")))?
        .map_err(AppError::from)?;

    // Sanitize project name for filename.
    let filename = sanitize_filename(
        &project_row
            .client_name
            .unwrap_or_else(|| "proposal".to_string()),
    );

    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"proposal-{filename}.pdf\""),
            ),
        ],
        pdf_bytes,
    ))
}

// ── Helpers ─────────────────────────────────────────────────

fn build_proposal_input(
    company_name: &str,
    project: &pt_repo::project::ProjectRow,
    zones: &[pt_project::Zone],
    materials: &[pt_materials::Material],
    good_quote: &pt_quote::Quote,
    better_quote: &pt_quote::Quote,
    best_quote: &pt_quote::Quote,
) -> pt_proposal::ProposalInput {
    let project_name = project
        .client_name
        .clone()
        .unwrap_or_else(|| "Untitled Project".to_string());
    let project_address = project
        .address
        .clone()
        .unwrap_or_else(|| "Address TBD".to_string());

    let make_tier_input = |quote: &pt_quote::Quote| -> pt_proposal::TierInput {
        let tier_level = format!("{:?}", quote.tier);
        let total = pt_proposal::format_dollars(quote.total);

        // Build zone summaries from the quote's line items.
        let mut zone_map: std::collections::HashMap<pt_project::ZoneId, pt_proposal::ZoneSummary> =
            std::collections::HashMap::new();

        for li in &quote.line_items {
            let zone = zones.iter().find(|z| z.id == li.zone_id);
            let entry = zone_map.entry(li.zone_id).or_insert_with(|| {
                let (label, zone_type, area) = match zone {
                    Some(z) => (
                        z.label.clone().unwrap_or_else(|| "Unnamed".to_string()),
                        format!("{:?}", z.zone_type),
                        pt_geo::area::area_sqft(&z.geometry),
                    ),
                    None => ("Unknown".to_string(), "Unknown".to_string(), 0.0),
                };
                pt_proposal::ZoneSummary {
                    label,
                    zone_type,
                    area_sqft: area,
                    materials: Vec::new(),
                }
            });
            // Look up material name from catalog.
            let mat_name = materials
                .iter()
                .find(|m| m.id == li.material_id)
                .map(|m| m.name.clone())
                .unwrap_or_else(|| li.material_name.clone());
            entry.materials.push(mat_name);
        }

        pt_proposal::TierInput {
            tier_level,
            total,
            zones: zone_map.into_values().collect(),
        }
    };

    pt_proposal::ProposalInput {
        company_name: company_name.to_string(),
        project_name,
        project_address,
        tiers: vec![
            make_tier_input(good_quote),
            make_tier_input(better_quote),
            make_tier_input(best_quote),
        ],
    }
}

fn extract_contact(contact: &Option<serde_json::Value>) -> (Option<String>, Option<String>) {
    match contact {
        Some(v) => {
            let phone = v.get("phone").and_then(|p| p.as_str()).map(String::from);
            let email = v.get("email").and_then(|e| e.as_str()).map(String::from);
            (phone, email)
        }
        None => (None, None),
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .to_lowercase()
}
