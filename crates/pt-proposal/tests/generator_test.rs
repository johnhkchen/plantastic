use pt_proposal::{
    MockFailingGenerator, MockProposalGenerator, ProposalError, ProposalInput,
    ProposalNarrativeGenerator, TierInput, ZoneSummary,
};
use pt_test_utils::timed;

fn sample_input() -> ProposalInput {
    ProposalInput {
        company_name: "Bay Area Landscapes".to_string(),
        project_name: "Johnson Backyard Refresh".to_string(),
        project_address: "742 Evergreen Terrace, San Jose, CA 95120".to_string(),
        tiers: vec![
            TierInput {
                tier_level: "Good".to_string(),
                total: "$4,250.00".to_string(),
                zones: vec![
                    ZoneSummary {
                        label: "Front Walkway".to_string(),
                        zone_type: "Hardscape".to_string(),
                        area_sqft: 120.0,
                        materials: vec![
                            "Concrete pavers".to_string(),
                            "Polymeric sand".to_string(),
                        ],
                    },
                    ZoneSummary {
                        label: "Patio".to_string(),
                        zone_type: "Hardscape".to_string(),
                        area_sqft: 180.0,
                        materials: vec!["Flagstone".to_string()],
                    },
                ],
            },
            TierInput {
                tier_level: "Better".to_string(),
                total: "$7,800.00".to_string(),
                zones: vec![ZoneSummary {
                    label: "Front Walkway".to_string(),
                    zone_type: "Hardscape".to_string(),
                    area_sqft: 120.0,
                    materials: vec!["Travertine pavers".to_string()],
                }],
            },
        ],
    }
}

#[test]
fn mock_returns_realistic_content() {
    timed(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let result = rt.block_on(MockProposalGenerator.generate(&sample_input()));
        let content = result.expect("mock should not fail");

        // Intro references company, project, and address
        assert!(
            content.intro_paragraph.contains("Bay Area Landscapes"),
            "intro should reference company name"
        );
        assert!(
            content.intro_paragraph.contains("Johnson Backyard Refresh"),
            "intro should reference project name"
        );
        assert!(
            content.intro_paragraph.contains("742 Evergreen Terrace"),
            "intro should reference address"
        );

        // Closing references company and project
        assert!(
            content.closing_paragraph.contains("Bay Area Landscapes"),
            "closing should reference company"
        );
        assert!(
            content
                .closing_paragraph
                .contains("Johnson Backyard Refresh"),
            "closing should reference project"
        );
    });
}

#[test]
fn mock_returns_tier_narratives_per_tier() {
    timed(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let content = rt
            .block_on(MockProposalGenerator.generate(&sample_input()))
            .unwrap();

        assert_eq!(
            content.tier_narratives.len(),
            2,
            "should have one narrative per input tier"
        );
        assert_eq!(content.tier_narratives[0].tier_level, "Good");
        assert_eq!(content.tier_narratives[1].tier_level, "Better");

        // Narrative text references zone labels
        assert!(
            content.tier_narratives[0]
                .description
                .contains("Front Walkway"),
            "Good tier narrative should mention zone labels"
        );
        assert!(
            content.tier_narratives[0].description.contains("Patio"),
            "Good tier narrative should mention all zones"
        );
    });
}

#[test]
fn mock_returns_zone_callouts() {
    timed(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let content = rt
            .block_on(MockProposalGenerator.generate(&sample_input()))
            .unwrap();

        // Zone callouts derived from first tier's zones
        assert_eq!(
            content.zone_callouts.len(),
            2,
            "should have one callout per zone in first tier"
        );
        assert_eq!(content.zone_callouts[0].zone_label, "Front Walkway");
        assert_eq!(content.zone_callouts[1].zone_label, "Patio");

        // Callout notes reference zone details
        assert!(
            content.zone_callouts[0].note.contains("120"),
            "callout should reference area"
        );
        assert!(
            content.zone_callouts[0].note.contains("Concrete pavers"),
            "callout should reference materials"
        );
    });
}

#[test]
fn mock_is_deterministic() {
    timed(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let input = sample_input();
        let a = rt.block_on(MockProposalGenerator.generate(&input)).unwrap();
        let b = rt.block_on(MockProposalGenerator.generate(&input)).unwrap();

        assert_eq!(a.intro_paragraph, b.intro_paragraph);
        assert_eq!(a.closing_paragraph, b.closing_paragraph);
        assert_eq!(a.tier_narratives.len(), b.tier_narratives.len());
        for (na, nb) in a.tier_narratives.iter().zip(b.tier_narratives.iter()) {
            assert_eq!(na.tier_level, nb.tier_level);
            assert_eq!(na.headline, nb.headline);
            assert_eq!(na.description, nb.description);
            assert_eq!(na.differentiators, nb.differentiators);
        }
        assert_eq!(a.zone_callouts.len(), b.zone_callouts.len());
        for (ca, cb) in a.zone_callouts.iter().zip(b.zone_callouts.iter()) {
            assert_eq!(ca.zone_label, cb.zone_label);
            assert_eq!(ca.note, cb.note);
        }
    });
}

#[test]
fn failing_mock_returns_error() {
    timed(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let result = rt.block_on(MockFailingGenerator.generate(&sample_input()));
        let err = result.expect_err("failing mock should return error");
        match err {
            ProposalError::Generation(msg) => {
                assert!(msg.contains("rate limit"), "error should describe failure");
            }
            other => panic!("expected Generation error, got: {other}"),
        }
    });
}

#[test]
fn mock_handles_empty_tiers() {
    timed(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let input = ProposalInput {
            company_name: "Test Co".to_string(),
            project_name: "Empty Project".to_string(),
            project_address: "123 Main St".to_string(),
            tiers: vec![],
        };
        let content = rt.block_on(MockProposalGenerator.generate(&input)).unwrap();

        assert!(content.tier_narratives.is_empty());
        assert!(content.zone_callouts.is_empty());
        assert!(!content.intro_paragraph.is_empty());
        assert!(!content.closing_paragraph.is_empty());
    });
}
