//! Integration tests for Typst PDF rendering.

use pt_materials::{MaterialId, Unit};
use pt_project::{TierLevel, ZoneId};
use pt_proposal::{
    render_proposal, ProposalContent, ProposalDocument, TenantBranding, TierNarrative, ZoneCallout,
};
use pt_quote::{LineItem, Quote};
use pt_test_utils::timed;
use rust_decimal::Decimal;
use std::str::FromStr;

fn mock_proposal_document() -> ProposalDocument {
    let zone_a = ZoneId::new();
    let zone_b = ZoneId::new();
    let mat_pavers = MaterialId::new();
    let mat_gravel = MaterialId::new();
    let mat_flagstone = MaterialId::new();
    let mat_decomposed = MaterialId::new();
    let mat_bluestone = MaterialId::new();
    let mat_pea_gravel = MaterialId::new();

    // Good tier: budget materials
    // Patio (12×15 = 180 sq ft): Standard Concrete Pavers @ $6.00 = $1,080.00
    // Garden Bed (10×20 = 200 sq ft): Pea Gravel @ $3.50 = $700.00
    // Subtotal: $1,780.00
    let good_quote = Quote {
        tier: TierLevel::Good,
        line_items: vec![
            LineItem {
                zone_id: zone_a,
                zone_label: Some("Back Patio".to_string()),
                material_id: mat_pavers,
                material_name: "Standard Concrete Pavers".to_string(),
                quantity: Decimal::from_str("180.00").unwrap(),
                unit: Unit::SqFt,
                unit_price: Decimal::from_str("6.00").unwrap(),
                line_total: Decimal::from_str("1080.00").unwrap(),
            },
            LineItem {
                zone_id: zone_b,
                zone_label: Some("Garden Bed".to_string()),
                material_id: mat_gravel,
                material_name: "Pea Gravel".to_string(),
                quantity: Decimal::from_str("200.00").unwrap(),
                unit: Unit::SqFt,
                unit_price: Decimal::from_str("3.50").unwrap(),
                line_total: Decimal::from_str("700.00").unwrap(),
            },
        ],
        subtotal: Decimal::from_str("1780.00").unwrap(),
        tax: None,
        total: Decimal::from_str("1780.00").unwrap(),
    };

    // Better tier: mid-range materials
    // Patio: Travertine Pavers @ $8.50 = $1,530.00
    // Garden Bed: Decomposed Granite @ $5.00 = $1,000.00
    // Subtotal: $2,530.00
    let better_quote = Quote {
        tier: TierLevel::Better,
        line_items: vec![
            LineItem {
                zone_id: zone_a,
                zone_label: Some("Back Patio".to_string()),
                material_id: mat_flagstone,
                material_name: "Travertine Pavers".to_string(),
                quantity: Decimal::from_str("180.00").unwrap(),
                unit: Unit::SqFt,
                unit_price: Decimal::from_str("8.50").unwrap(),
                line_total: Decimal::from_str("1530.00").unwrap(),
            },
            LineItem {
                zone_id: zone_b,
                zone_label: Some("Garden Bed".to_string()),
                material_id: mat_decomposed,
                material_name: "Decomposed Granite".to_string(),
                quantity: Decimal::from_str("200.00").unwrap(),
                unit: Unit::SqFt,
                unit_price: Decimal::from_str("5.00").unwrap(),
                line_total: Decimal::from_str("1000.00").unwrap(),
            },
        ],
        subtotal: Decimal::from_str("2530.00").unwrap(),
        tax: None,
        total: Decimal::from_str("2530.00").unwrap(),
    };

    // Best tier: premium materials
    // Patio: Natural Bluestone @ $14.00 = $2,520.00
    // Garden Bed: Premium Pea Gravel @ $7.50 = $1,500.00
    // Subtotal: $4,020.00
    let best_quote = Quote {
        tier: TierLevel::Best,
        line_items: vec![
            LineItem {
                zone_id: zone_a,
                zone_label: Some("Back Patio".to_string()),
                material_id: mat_bluestone,
                material_name: "Natural Bluestone".to_string(),
                quantity: Decimal::from_str("180.00").unwrap(),
                unit: Unit::SqFt,
                unit_price: Decimal::from_str("14.00").unwrap(),
                line_total: Decimal::from_str("2520.00").unwrap(),
            },
            LineItem {
                zone_id: zone_b,
                zone_label: Some("Garden Bed".to_string()),
                material_id: mat_pea_gravel,
                material_name: "Premium Pea Gravel".to_string(),
                quantity: Decimal::from_str("200.00").unwrap(),
                unit: Unit::SqFt,
                unit_price: Decimal::from_str("7.50").unwrap(),
                line_total: Decimal::from_str("1500.00").unwrap(),
            },
        ],
        subtotal: Decimal::from_str("4020.00").unwrap(),
        tax: None,
        total: Decimal::from_str("4020.00").unwrap(),
    };

    let narrative = ProposalContent {
        intro_paragraph: "Thank you for choosing Green Valley Landscapes for your \
            Sunset Garden project at 123 Oak Street, San Jose, CA. We are pleased \
            to present this comprehensive landscape proposal."
            .to_string(),
        tier_narratives: vec![
            TierNarrative {
                tier_level: "Good".to_string(),
                headline: "Good Package for Sunset Garden".to_string(),
                description: "A solid foundation with quality budget-friendly materials."
                    .to_string(),
                differentiators: vec![
                    "Covers 2 distinct zones".to_string(),
                    "All-inclusive $1,780.00 pricing".to_string(),
                    "Professional installation and cleanup".to_string(),
                ],
            },
            TierNarrative {
                tier_level: "Better".to_string(),
                headline: "Better Package for Sunset Garden".to_string(),
                description: "Upgraded materials for a polished, lasting look.".to_string(),
                differentiators: vec![
                    "Premium travertine and granite".to_string(),
                    "All-inclusive $2,530.00 pricing".to_string(),
                    "Enhanced curb appeal".to_string(),
                ],
            },
            TierNarrative {
                tier_level: "Best".to_string(),
                headline: "Best Package for Sunset Garden".to_string(),
                description: "Top-tier natural stone for a show-stopping landscape.".to_string(),
                differentiators: vec![
                    "Natural bluestone and premium gravel".to_string(),
                    "All-inclusive $4,020.00 pricing".to_string(),
                    "Heirloom-quality craftsmanship".to_string(),
                ],
            },
        ],
        zone_callouts: vec![
            ZoneCallout {
                zone_label: "Back Patio".to_string(),
                note: "180 sq ft entertaining area with durable paving materials.".to_string(),
            },
            ZoneCallout {
                zone_label: "Garden Bed".to_string(),
                note: "200 sq ft planting bed with decorative ground cover.".to_string(),
            },
        ],
        closing_paragraph: "We at Green Valley Landscapes look forward to bringing \
            your vision for Sunset Garden to life. Please don't hesitate to reach \
            out with any questions."
            .to_string(),
    };

    let branding = TenantBranding {
        company_name: "Green Valley Landscapes".to_string(),
        logo_url: None,
        primary_color: Some("#2E7D32".to_string()),
        phone: Some("(408) 555-0123".to_string()),
        email: Some("info@greenvalley.example.com".to_string()),
    };

    ProposalDocument {
        project_name: "Sunset Garden".to_string(),
        project_address: "123 Oak Street, San Jose, CA 95125".to_string(),
        date: "April 1, 2026".to_string(),
        branding,
        narrative,
        good_quote,
        better_quote,
        best_quote,
    }
}

#[test]
fn render_produces_valid_pdf() {
    timed(|| {
        let doc = mock_proposal_document();
        let pdf_bytes = render_proposal(&doc).expect("render_proposal should succeed");

        // Valid PDF starts with the %PDF- magic bytes.
        assert!(
            pdf_bytes.starts_with(b"%PDF-"),
            "output should be a valid PDF (starts with %PDF-), got: {:?}",
            &pdf_bytes[..pdf_bytes.len().min(20)]
        );
    });
}

#[test]
fn render_pdf_size_reasonable() {
    timed(|| {
        let doc = mock_proposal_document();
        let pdf_bytes = render_proposal(&doc).expect("render_proposal should succeed");

        assert!(
            pdf_bytes.len() > 10_000,
            "PDF should be >10KB, was {} bytes",
            pdf_bytes.len()
        );
        assert!(
            pdf_bytes.len() < 5_000_000,
            "PDF should be <5MB, was {} bytes",
            pdf_bytes.len()
        );
    });
}
