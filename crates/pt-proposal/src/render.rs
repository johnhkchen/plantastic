//! Typst-based PDF rendering for branded proposals.
//!
//! Merges three-tier quotes, BAML narrative content, and tenant branding
//! into a professional PDF via an embedded Typst template.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use pt_materials::Unit;

use crate::error::ProposalError;
use crate::ProposalContent;
use pt_quote::Quote;

/// Tenant branding fields for the proposal header and styling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantBranding {
    pub company_name: String,
    pub logo_url: Option<String>,
    /// Hex color for accent elements, e.g. `"#4CAF50"`.
    pub primary_color: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

/// Complete input for PDF rendering: quotes + narrative + branding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalDocument {
    pub project_name: String,
    pub project_address: String,
    /// Pre-formatted date string, e.g. `"April 1, 2026"`.
    pub date: String,
    pub branding: TenantBranding,
    pub narrative: ProposalContent,
    pub good_quote: Quote,
    pub better_quote: Quote,
    pub best_quote: Quote,
}

// ── Private: template data with pre-formatted dollar amounts ──────

#[derive(Serialize)]
struct TemplateData {
    project_name: String,
    project_address: String,
    date: String,
    branding: TenantBranding,
    intro_paragraph: String,
    closing_paragraph: String,
    tiers: Vec<TemplateTier>,
    zone_callouts: Vec<TemplateZoneCallout>,
    tier_narratives: Vec<TemplateTierNarrative>,
}

#[derive(Serialize)]
struct TemplateTier {
    level: String,
    line_items: Vec<TemplateLineItem>,
    subtotal: String,
    tax: Option<String>,
    total: String,
}

#[derive(Serialize)]
struct TemplateLineItem {
    zone_label: String,
    material_name: String,
    quantity: String,
    unit: String,
    unit_price: String,
    line_total: String,
}

#[derive(Serialize)]
struct TemplateZoneCallout {
    zone_label: String,
    note: String,
}

#[derive(Serialize)]
struct TemplateTierNarrative {
    tier_level: String,
    headline: String,
    description: String,
    differentiators: Vec<String>,
}

// ── Dollar formatting ────────────────────────────────────────────

fn format_dollars(d: Decimal) -> String {
    let is_negative = d < Decimal::ZERO;
    let abs = if is_negative { -d } else { d };
    // Round to 2 decimal places.
    let rounded = abs.round_dp(2);
    let s = rounded.to_string();

    // Split on decimal point.
    let (int_part, dec_part) = match s.split_once('.') {
        Some((i, d)) => (i.to_string(), format!("{d:0<2}")),
        None => (s.clone(), "00".to_string()),
    };

    // Add thousand separators.
    let int_bytes: Vec<char> = int_part.chars().collect();
    let mut formatted = String::new();
    for (i, ch) in int_bytes.iter().enumerate() {
        if i > 0 && (int_bytes.len() - i) % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(*ch);
    }

    if is_negative {
        format!("-${formatted}.{dec_part}")
    } else {
        format!("${formatted}.{dec_part}")
    }
}

fn format_unit(unit: &Unit) -> &'static str {
    match unit {
        Unit::SqFt => "sq ft",
        Unit::CuYd => "cu yd",
        Unit::LinearFt => "lin ft",
        Unit::Each => "ea",
    }
}

fn quote_to_template_tier(q: &Quote) -> TemplateTier {
    let level = format!("{:?}", q.tier);
    let line_items = q
        .line_items
        .iter()
        .map(|li| TemplateLineItem {
            zone_label: li.zone_label.clone().unwrap_or_else(|| "—".to_string()),
            material_name: li.material_name.clone(),
            quantity: li.quantity.round_dp(2).to_string(),
            unit: format_unit(&li.unit).to_string(),
            unit_price: format_dollars(li.unit_price),
            line_total: format_dollars(li.line_total),
        })
        .collect();

    TemplateTier {
        level,
        line_items,
        subtotal: format_dollars(q.subtotal),
        tax: q.tax.map(format_dollars),
        total: format_dollars(q.total),
    }
}

fn to_template_data(doc: &ProposalDocument) -> TemplateData {
    TemplateData {
        project_name: doc.project_name.clone(),
        project_address: doc.project_address.clone(),
        date: doc.date.clone(),
        branding: doc.branding.clone(),
        intro_paragraph: doc.narrative.intro_paragraph.clone(),
        closing_paragraph: doc.narrative.closing_paragraph.clone(),
        tiers: vec![
            quote_to_template_tier(&doc.good_quote),
            quote_to_template_tier(&doc.better_quote),
            quote_to_template_tier(&doc.best_quote),
        ],
        zone_callouts: doc
            .narrative
            .zone_callouts
            .iter()
            .map(|zc| TemplateZoneCallout {
                zone_label: zc.zone_label.clone(),
                note: zc.note.clone(),
            })
            .collect(),
        tier_narratives: doc
            .narrative
            .tier_narratives
            .iter()
            .map(|tn| TemplateTierNarrative {
                tier_level: tn.tier_level.clone(),
                headline: tn.headline.clone(),
                description: tn.description.clone(),
                differentiators: tn.differentiators.clone(),
            })
            .collect(),
    }
}

// ── Embedded template ────────────────────────────────────────────

static TEMPLATE: &str = include_str!("../templates/proposal.typ");

// ── Public API ───────────────────────────────────────────────────

/// Render a branded proposal PDF from quotes, narrative, and branding.
///
/// Returns the raw PDF bytes on success.
pub fn render_proposal(data: ProposalDocument) -> Result<Vec<u8>, ProposalError> {
    use typst_as_lib::TypstTemplate;

    let template_data = to_template_data(&data);
    let json_str =
        serde_json::to_string(&template_data).map_err(|e| ProposalError::Render(e.to_string()))?;

    let template = TypstTemplate::new(vec![], TEMPLATE);
    let doc = template
        .compile_with_input(&[("data".to_string(), json_str.into())])
        .map_err(|e| ProposalError::Render(format!("{e:?}")))?;
    let pdf_bytes = typst_as_lib::export_pdf(&doc)
        .map_err(|e| ProposalError::Render(format!("{e:?}")))?;

    Ok(pdf_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_dollars_basic() {
        assert_eq!(format_dollars(Decimal::new(153000, 2)), "$1,530.00");
    }

    #[test]
    fn format_dollars_small() {
        assert_eq!(format_dollars(Decimal::new(850, 2)), "$8.50");
    }

    #[test]
    fn format_dollars_large() {
        assert_eq!(format_dollars(Decimal::new(1234567_89, 2)), "$1,234,567.89");
    }

    #[test]
    fn format_dollars_zero() {
        assert_eq!(format_dollars(Decimal::ZERO), "$0.00");
    }
}
