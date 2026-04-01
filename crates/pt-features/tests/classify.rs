//! Integration tests for feature classification.

use pt_features::{
    ClassificationError, FeatureClassifier, MockFailingClassifier, MockFeatureClassifier,
};
use pt_scan::FeatureCandidate;

fn tree_trunk_candidate(id: usize) -> FeatureCandidate {
    FeatureCandidate {
        cluster_id: id,
        centroid: [-3.21, 0.42, 3.81],
        bbox_min: [-3.8, -0.2, 0.0],
        bbox_max: [-2.6, 1.0, 7.6],
        height_ft: 25.1,
        spread_ft: 2.8,
        point_count: 1247,
        dominant_color: "brown".to_string(),
        vertical_profile: "columnar".to_string(),
        density: 892.3,
    }
}

fn curb_candidate(id: usize) -> FeatureCandidate {
    FeatureCandidate {
        cluster_id: id,
        centroid: [1.0, 2.0, 0.1],
        bbox_min: [0.0, 1.5, 0.0],
        bbox_max: [2.0, 2.5, 0.15],
        height_ft: 0.5,
        spread_ft: 6.5,
        point_count: 340,
        dominant_color: "gray".to_string(),
        vertical_profile: "flat".to_string(),
        density: 4500.0,
    }
}

fn utility_pole_candidate(id: usize) -> FeatureCandidate {
    FeatureCandidate {
        cluster_id: id,
        centroid: [5.0, 3.0, 5.0],
        bbox_min: [4.8, 2.8, 0.0],
        bbox_max: [5.2, 3.2, 10.0],
        height_ft: 32.8,
        spread_ft: 1.3,
        point_count: 520,
        dominant_color: "gray".to_string(),
        vertical_profile: "columnar".to_string(),
        density: 325.0,
    }
}

#[tokio::test]
async fn test_mock_classifies_tree_trunk() {
    let classifier = MockFeatureClassifier;
    let candidates = vec![tree_trunk_candidate(0)];
    let results = classifier
        .classify(&candidates, "Powell & Market, SF", "USDA 10b")
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    let r = &results[0];
    assert_eq!(r.cluster_id, 0);
    assert_eq!(r.category, "tree");
    assert!(r.confidence > 0.7, "tree confidence={}", r.confidence);
    assert!(r.species.is_some(), "tree should have species");
    assert!(!r.reasoning.is_empty());
    assert!(!r.landscape_notes.is_empty());
}

#[tokio::test]
async fn test_mock_classifies_hardscape() {
    let classifier = MockFeatureClassifier;
    let candidates = vec![curb_candidate(1)];
    let results = classifier
        .classify(&candidates, "Powell & Market, SF", "USDA 10b")
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    let r = &results[0];
    assert_eq!(r.cluster_id, 1);
    assert_eq!(r.category, "hardscape");
    assert!(r.species.is_none(), "hardscape should not have species");
}

#[tokio::test]
async fn test_mock_classifies_utility_pole() {
    let classifier = MockFeatureClassifier;
    let candidates = vec![utility_pole_candidate(2)];
    let results = classifier
        .classify(&candidates, "Powell & Market, SF", "USDA 10b")
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    let r = &results[0];
    assert_eq!(r.cluster_id, 2);
    assert_eq!(r.category, "utility");
    assert!(r.species.is_none());
}

#[tokio::test]
async fn test_mock_classifies_mixed_candidates() {
    let classifier = MockFeatureClassifier;
    let candidates = vec![
        tree_trunk_candidate(0),
        curb_candidate(1),
        utility_pole_candidate(2),
    ];
    let results = classifier
        .classify(&candidates, "Powell & Market, SF", "USDA 10b")
        .await
        .unwrap();

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].cluster_id, 0);
    assert_eq!(results[1].cluster_id, 1);
    assert_eq!(results[2].cluster_id, 2);

    assert_eq!(results[0].category, "tree");
    assert_eq!(results[1].category, "hardscape");
    assert_eq!(results[2].category, "utility");
}

#[tokio::test]
async fn test_mock_powell_market_two_trees() {
    // Simulate the two main tree trunk clusters from Powell & Market scan
    let candidates = vec![
        FeatureCandidate {
            cluster_id: 0,
            centroid: [-3.21, 0.42, 3.81],
            bbox_min: [-3.8, -0.2, 0.0],
            bbox_max: [-2.6, 1.0, 7.6],
            height_ft: 25.1,
            spread_ft: 2.8,
            point_count: 1247,
            dominant_color: "brown".to_string(),
            vertical_profile: "columnar".to_string(),
            density: 892.3,
        },
        FeatureCandidate {
            cluster_id: 1,
            centroid: [1.85, -0.31, 3.44],
            bbox_min: [1.2, -0.9, 0.0],
            bbox_max: [2.5, 0.3, 6.9],
            height_ft: 22.6,
            spread_ft: 2.3,
            point_count: 983,
            dominant_color: "brown".to_string(),
            vertical_profile: "columnar".to_string(),
            density: 756.1,
        },
    ];

    let classifier = MockFeatureClassifier;
    let results = classifier
        .classify(
            &candidates,
            "Powell & Market Streets, San Francisco, CA",
            "USDA 10b / Sunset 17 — Mediterranean maritime",
        )
        .await
        .unwrap();

    assert_eq!(results.len(), 2);

    for (i, r) in results.iter().enumerate() {
        assert_eq!(
            r.category, "tree",
            "candidate {i} should be tree, got {}",
            r.category
        );
        assert!(
            r.confidence > 0.7,
            "candidate {i} confidence={}, expected > 0.7",
            r.confidence
        );
        assert!(
            r.species.is_some(),
            "candidate {i} should have species identification"
        );
        assert!(
            !r.reasoning.is_empty(),
            "candidate {i} should have reasoning"
        );
        assert!(
            !r.landscape_notes.is_empty(),
            "candidate {i} should have landscape notes"
        );
    }
}

#[tokio::test]
async fn test_mock_empty_input() {
    let classifier = MockFeatureClassifier;
    let results = classifier
        .classify(&[], "anywhere", "any zone")
        .await
        .unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_failing_classifier() {
    let classifier = MockFailingClassifier;
    let result = classifier
        .classify(&[tree_trunk_candidate(0)], "anywhere", "any zone")
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, ClassificationError::Classification(_)),
        "expected Classification error, got {err:?}"
    );
}

#[tokio::test]
async fn test_classified_feature_fields_valid() {
    let classifier = MockFeatureClassifier;
    let candidates = vec![
        tree_trunk_candidate(0),
        curb_candidate(1),
        utility_pole_candidate(2),
    ];
    let results = classifier
        .classify(&candidates, "test", "test")
        .await
        .unwrap();

    let valid_categories = ["tree", "structure", "hardscape", "planting", "utility"];
    for r in &results {
        assert!(
            valid_categories.contains(&r.category.as_str()),
            "invalid category: {}",
            r.category
        );
        assert!(
            (0.0..=1.0).contains(&r.confidence),
            "confidence out of range: {}",
            r.confidence
        );
        assert!(!r.label.is_empty(), "label should not be empty");
        assert!(!r.reasoning.is_empty(), "reasoning should not be empty");
        assert!(
            !r.landscape_notes.is_empty(),
            "landscape_notes should not be empty"
        );
    }
}
