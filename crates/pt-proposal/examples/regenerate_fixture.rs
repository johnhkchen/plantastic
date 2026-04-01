//! Regenerate mock fixtures from real LLM output.
//!
//! Calls `ClaudeCliGenerator` (subscription, zero API cost) with the
//! canonical sample input and writes the result to `src/fixtures/sample_output.json`.
//!
//! Usage:
//!   cargo run -p pt-proposal --example regenerate_fixture
//!
//! After running, review the diff and commit the updated fixture:
//!   git diff crates/pt-proposal/src/fixtures/sample_output.json

use pt_proposal::{
    ClaudeCliGenerator, ProposalInput, ProposalNarrativeGenerator, TierInput, ZoneSummary,
};
use std::path::PathBuf;

/// The canonical sample input. Matches `fixtures/sample_input.json`.
/// If you change this, update `sample_input.json` too.
fn sample_input() -> ProposalInput {
    ProposalInput {
        company_name: "Bay Area Landscapes".into(),
        project_name: "Johnson Backyard Refresh".into(),
        project_address: "1234 Oak Ave, San Francisco, CA".into(),
        tiers: vec![
            TierInput {
                tier_level: "Good".into(),
                total: "$4,230.00".into(),
                zones: vec![
                    ZoneSummary {
                        label: "Main Patio".into(),
                        zone_type: "patio".into(),
                        area_sqft: 180.0,
                        materials: vec!["Concrete Pavers".into()],
                    },
                    ZoneSummary {
                        label: "Front Bed".into(),
                        zone_type: "bed".into(),
                        area_sqft: 160.0,
                        materials: vec!["Standard Mulch".into()],
                    },
                ],
            },
            TierInput {
                tier_level: "Better".into(),
                total: "$6,890.00".into(),
                zones: vec![
                    ZoneSummary {
                        label: "Main Patio".into(),
                        zone_type: "patio".into(),
                        area_sqft: 180.0,
                        materials: vec!["Travertine Pavers".into()],
                    },
                    ZoneSummary {
                        label: "Front Bed".into(),
                        zone_type: "bed".into(),
                        area_sqft: 160.0,
                        materials: vec!["Premium Mulch".into(), "Steel Edging".into()],
                    },
                ],
            },
            TierInput {
                tier_level: "Best".into(),
                total: "$11,450.00".into(),
                zones: vec![
                    ZoneSummary {
                        label: "Main Patio".into(),
                        zone_type: "patio".into(),
                        area_sqft: 180.0,
                        materials: vec!["Flagstone".into()],
                    },
                    ZoneSummary {
                        label: "Front Bed".into(),
                        zone_type: "bed".into(),
                        area_sqft: 160.0,
                        materials: vec![
                            "Cedar Bark".into(),
                            "Corten Steel Edging".into(),
                            "Landscape Lighting".into(),
                        ],
                    },
                ],
            },
        ],
    }
}

#[tokio::main]
async fn main() {
    let input = sample_input();
    let generator = ClaudeCliGenerator;

    println!("Generating proposal via claude CLI (subscription)...");

    let content = generator.generate(&input).await.unwrap_or_else(|e| {
        eprintln!("Generation failed: {e}");
        std::process::exit(1);
    });

    // Serialize to pretty JSON. We use serde_json on the raw fields since
    // BAML types don't derive Serialize. Build a serde_json::Value manually.
    let json = serde_json::json!({
        "intro_paragraph": content.intro_paragraph,
        "tier_narratives": content.tier_narratives.iter().map(|t| serde_json::json!({
            "tier_level": t.tier_level,
            "headline": t.headline,
            "description": t.description,
            "differentiators": t.differentiators,
        })).collect::<Vec<_>>(),
        "zone_callouts": content.zone_callouts.iter().map(|z| serde_json::json!({
            "zone_label": z.zone_label,
            "note": z.note,
        })).collect::<Vec<_>>(),
        "closing_paragraph": content.closing_paragraph,
    });

    let pretty = serde_json::to_string_pretty(&json).unwrap();

    // Write to fixture file
    let fixture_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/fixtures/sample_output.json");

    let old_content = std::fs::read_to_string(&fixture_path).unwrap_or_default();

    std::fs::write(&fixture_path, format!("{pretty}\n")).unwrap_or_else(|e| {
        eprintln!("Failed to write {}: {e}", fixture_path.display());
        std::process::exit(1);
    });

    println!("Wrote: {}", fixture_path.display());

    if old_content.trim() == pretty.trim() {
        println!("No changes — fixture is already up to date.");
    } else {
        println!("Fixture updated. Review with:");
        println!("  git diff {}", fixture_path.display());
    }
}
