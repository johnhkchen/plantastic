//! Mock implementation of [`SiteReconciler`] for tests.

use async_trait::async_trait;

use crate::baml_client::types::{Discrepancy, RecommendedZone, ReconciledFeature, ReconciledSite};
use crate::error::ReconcilerError;
use crate::reconciler::{ReconcilerInput, SiteReconciler};

/// Returns deterministic reconciled site data for Powell & Market.
///
/// Two confirmed London Planes, one scan-only fire hydrant, one
/// satellite-only removed tree, one height discrepancy, and four
/// recommended zones informed by all data sources.
#[derive(Debug)]
pub struct MockSiteReconciler;

#[async_trait]
impl SiteReconciler for MockSiteReconciler {
    async fn reconcile(&self, _input: &ReconcilerInput) -> Result<ReconciledSite, ReconcilerError> {
        Ok(powell_market_fixture())
    }
}

/// Always returns a reconciliation error. Use for testing error-handling paths.
#[derive(Debug)]
pub struct MockFailingReconciler;

#[async_trait]
impl SiteReconciler for MockFailingReconciler {
    async fn reconcile(&self, _input: &ReconcilerInput) -> Result<ReconciledSite, ReconcilerError> {
        Err(ReconcilerError::Reconciliation(
            "mock LLM failure: rate limit exceeded".to_string(),
        ))
    }
}

/// Hand-crafted fixture for Powell & Market Streets reconciliation.
///
/// Scenario: 2 London Planes confirmed (scan + satellite), 1 fire hydrant
/// scan-only, 1 satellite tree not found in scan (possible removal),
/// 1 height discrepancy between sources, 4 recommended zones.
pub fn powell_market_fixture() -> ReconciledSite {
    ReconciledSite {
        confirmed_features: vec![
            ReconciledFeature {
                label: "London Plane Tree #1".to_string(),
                category: "tree".to_string(),
                source: "both".to_string(),
                confidence: 0.85,
                reasoning: "Scan shows 25.1 ft columnar trunk matching satellite \
                    tree #1 (30.0 ft canopy height). Height difference is expected: \
                    scan measures trunk only, satellite measures canopy top."
                    .to_string(),
            },
            ReconciledFeature {
                label: "London Plane Tree #2".to_string(),
                category: "tree".to_string(),
                source: "both".to_string(),
                confidence: 0.82,
                reasoning: "Scan shows 22.6 ft columnar trunk matching satellite \
                    tree #2 (28.0 ft canopy height). Consistent species and location."
                    .to_string(),
            },
        ],
        scan_only_features: vec![ReconciledFeature {
            label: "Fire Hydrant".to_string(),
            category: "utility".to_string(),
            source: "scan".to_string(),
            confidence: 0.91,
            reasoning: "Small ground-level utility not detectable by satellite \
                canopy analysis. 3 ft municipal clearance required."
                .to_string(),
        }],
        satellite_only_features: vec![ReconciledFeature {
            label: "Removed Tree (former position)".to_string(),
            category: "tree".to_string(),
            source: "satellite".to_string(),
            confidence: 0.55,
            reasoning: "Satellite detected a third tree at 20.0 ft height with 10.0 ft \
                spread, but LiDAR scan shows no corresponding feature. The tree was \
                likely removed between the satellite image capture and the site visit."
                .to_string(),
        }],
        discrepancies: vec![Discrepancy {
            description: "London Plane #1 height: scan measures 25.1 ft, satellite \
                reports 30.0 ft."
                .to_string(),
            possible_explanation: "Scan captures trunk height (LiDAR line-of-sight \
                from ground level), while satellite canopy model includes full crown. \
                The ~5 ft difference is consistent with a mature London Plane crown spread."
                .to_string(),
            design_implication: "Use the satellite canopy height (30 ft) for shade \
                modeling and the scan trunk position for hardscape setback calculations."
                .to_string(),
        }],
        recommended_zones: vec![
            RecommendedZone {
                label: "Entry Patio".to_string(),
                zone_type: "patio".to_string(),
                rationale: "The clear area between the two confirmed London Planes is \
                    the natural focal point. Satellite sun data shows 8.5 hours average \
                    exposure, but canopy shade from both trees reduces effective exposure \
                    to ~4 hours — suitable for a dappled-light patio with permeable pavers."
                    .to_string(),
                approximate_area_sqft: 18.4,
                sun_exposure_hours: Some(4),
                data_sources: "scan, satellite, plan_view".to_string(),
            },
            RecommendedZone {
                label: "East Shade Garden".to_string(),
                zone_type: "bed".to_string(),
                rationale: "Confirmed by both plan-view analysis and satellite shade \
                    modeling. The east side of London Plane #1 receives afternoon shade \
                    from the 30 ft canopy. Shade-loving perennials (Heuchera, Japanese \
                    Forest Grass) would thrive with minimal irrigation."
                    .to_string(),
                approximate_area_sqft: 24.0,
                sun_exposure_hours: Some(3),
                data_sources: "scan, satellite, plan_view".to_string(),
            },
            RecommendedZone {
                label: "Removal Site Rain Garden".to_string(),
                zone_type: "bed".to_string(),
                rationale: "The satellite-only tree removal site presents an opportunity. \
                    The existing root zone may have altered soil drainage. A rain garden \
                    at the former tree location would handle stormwater runoff while the \
                    disturbed soil settles. Full sun exposure (~8.5 hours) supports \
                    sun-loving native species."
                    .to_string(),
                approximate_area_sqft: 15.0,
                sun_exposure_hours: Some(8),
                data_sources: "satellite, plan_view".to_string(),
            },
            RecommendedZone {
                label: "Street Buffer Planting".to_string(),
                zone_type: "bed".to_string(),
                rationale: "Confirmed by plan-view observations about pedestrian traffic. \
                    A low planting strip along the curb softens the hardscape boundary. \
                    Fire hydrant clearance zone constrains the western extent. Dymondia \
                    or Carex pansa would tolerate foot traffic and partial shade."
                    .to_string(),
                approximate_area_sqft: 28.0,
                sun_exposure_hours: Some(6),
                data_sources: "scan, satellite, plan_view".to_string(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::baml_client::types::{
        ClassifiedFeature, SatelliteBaseline, SatelliteTree, SiteAnalysis,
    };
    use pt_test_utils::timed;

    fn test_input() -> ReconcilerInput {
        ReconcilerInput {
            scan_features: vec![ClassifiedFeature {
                cluster_id: 0,
                label: "London Plane Tree".to_string(),
                category: "tree".to_string(),
                species: Some("Platanus × acerifolia".to_string()),
                confidence: 0.85,
                reasoning: "25.1 ft columnar trunk.".to_string(),
                landscape_notes: "Root zone setback required.".to_string(),
            }],
            satellite_baseline: SatelliteBaseline {
                lot_area_sqft: 1000.0,
                trees: vec![SatelliteTree {
                    height_ft: 30.0,
                    spread_ft: 15.0,
                    confidence: 0.78,
                }],
                avg_sun_hours: 8.5,
            },
            plan_view_analysis: SiteAnalysis {
                features: vec![],
                suggested_zones: vec![],
                site_observations: vec![],
            },
            address: "Powell & Market Streets, San Francisco, CA".to_string(),
        }
    }

    #[test]
    fn fixture_has_expected_counts() {
        timed(|| {
            let site = powell_market_fixture();
            // 2 confirmed (both London Planes), hand-counted from fixture above.
            assert_eq!(site.confirmed_features.len(), 2);
            // 1 scan-only (fire hydrant).
            assert_eq!(site.scan_only_features.len(), 1);
            // 1 satellite-only (removed tree).
            assert_eq!(site.satellite_only_features.len(), 1);
            // 1 discrepancy (height mismatch).
            assert_eq!(site.discrepancies.len(), 1);
            // 4 recommended zones.
            assert_eq!(site.recommended_zones.len(), 4);
        });
    }

    #[test]
    fn confirmed_features_have_both_source() {
        timed(|| {
            let site = powell_market_fixture();
            for f in &site.confirmed_features {
                assert_eq!(
                    f.source, "both",
                    "confirmed feature should have source 'both'"
                );
            }
        });
    }

    #[test]
    fn scan_only_features_have_scan_source() {
        timed(|| {
            let site = powell_market_fixture();
            for f in &site.scan_only_features {
                assert_eq!(f.source, "scan");
            }
        });
    }

    #[test]
    fn satellite_only_features_have_satellite_source() {
        timed(|| {
            let site = powell_market_fixture();
            for f in &site.satellite_only_features {
                assert_eq!(f.source, "satellite");
            }
        });
    }

    #[tokio::test]
    async fn mock_reconciler_returns_fixture() {
        let reconciler = MockSiteReconciler;
        let result = reconciler.reconcile(&test_input()).await;
        assert!(result.is_ok());
        let site = result.unwrap();
        assert_eq!(site.confirmed_features.len(), 2);
        assert_eq!(site.recommended_zones.len(), 4);
    }

    #[tokio::test]
    async fn mock_failing_reconciler_returns_error() {
        let reconciler = MockFailingReconciler;
        let result = reconciler.reconcile(&test_input()).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("rate limit exceeded"));
    }
}
