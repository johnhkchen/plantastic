//! Test proposal generation via the `claude` CLI (subscription, zero API cost).
//!
//! Usage:
//!   cargo run -p pt-proposal --example test_proposal

use pt_proposal::{
    ClaudeCliGenerator, ProposalContent, ProposalInput, ProposalNarrativeGenerator, TierInput,
    ZoneSummary,
};

fn test_input() -> ProposalInput {
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

fn print_proposal(c: &ProposalContent) {
    println!("=== PROPOSAL NARRATIVE ===\n");
    println!("INTRO:\n{}\n", c.intro_paragraph);
    for t in &c.tier_narratives {
        println!("--- {} ---", t.tier_level);
        println!("Headline: {}", t.headline);
        println!("Description: {}", t.description);
        println!("Differentiators:");
        for d in &t.differentiators {
            println!("  • {d}");
        }
        println!();
    }
    println!("ZONE CALLOUTS:");
    for z in &c.zone_callouts {
        println!("  [{}] {}", z.zone_label, z.note);
    }
    println!("\nCLOSING:\n{}", c.closing_paragraph);
}

#[tokio::main]
async fn main() {
    let input = test_input();
    let generator = ClaudeCliGenerator;

    println!("Generating proposal via claude CLI (subscription)...\n");

    match generator.generate(&input).await {
        Ok(content) => print_proposal(&content),
        Err(e) => {
            eprintln!("Failed: {e}");
            std::process::exit(1);
        }
    }
}
