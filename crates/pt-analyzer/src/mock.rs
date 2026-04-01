//! Mock implementation of [`SiteAnalyzer`] for tests.

use async_trait::async_trait;

use crate::analyzer::{SiteAnalyzer, SiteAnalyzerInput};
use crate::baml_client::types::{ClassifiedFeature, SiteAnalysis, SiteObservation, SuggestedZone};
use crate::error::AnalyzerError;

/// Returns deterministic site analysis for Powell & Market.
///
/// Four suggested zones and three site observations, appropriate for
/// the SF urban streetscape scenario. Echoes back classified features
/// from the input.
#[derive(Debug)]
pub struct MockSiteAnalyzer;

#[async_trait]
impl SiteAnalyzer for MockSiteAnalyzer {
    async fn analyze(&self, input: &SiteAnalyzerInput) -> Result<SiteAnalysis, AnalyzerError> {
        Ok(powell_market_fixture(&input.classified_features))
    }
}

/// Always returns an analysis error. Use for testing error-handling paths.
#[derive(Debug)]
pub struct MockFailingAnalyzer;

#[async_trait]
impl SiteAnalyzer for MockFailingAnalyzer {
    async fn analyze(&self, _input: &SiteAnalyzerInput) -> Result<SiteAnalysis, AnalyzerError> {
        Err(AnalyzerError::Analysis(
            "mock LLM failure: rate limit exceeded".to_string(),
        ))
    }
}

/// Hand-crafted fixture for Powell & Market Streets site analysis.
///
/// Two London Plane trunks visible in plan view. The analysis suggests
/// zones that reference spatial relationships between features, and
/// observations that demonstrate landscape design expertise.
fn powell_market_fixture(classified_features: &[ClassifiedFeature]) -> SiteAnalysis {
    SiteAnalysis {
        features: classified_features.to_vec(),
        suggested_zones: vec![
            SuggestedZone {
                label: "Entry Patio".to_string(),
                zone_type: "patio".to_string(),
                rationale: "The clear area between the two London Plane trunks is \
                    the natural focal point of the frontage. A permeable paver patio \
                    here creates a welcoming transition from sidewalk to building entry."
                    .to_string(),
                approximate_area_sqft: 18.4,
            },
            SuggestedZone {
                label: "East Shade Garden".to_string(),
                zone_type: "bed".to_string(),
                rationale: "The east side of the larger London Plane receives afternoon \
                    shade from the tree canopy. Shade-loving perennials would thrive here \
                    with minimal irrigation."
                    .to_string(),
                approximate_area_sqft: 24.0,
            },
            SuggestedZone {
                label: "Street Buffer Planting".to_string(),
                zone_type: "bed".to_string(),
                rationale: "A low planting strip along the curb edge softens the \
                    hardscape boundary and provides visual screening from street traffic \
                    without blocking sightlines to the building entrance."
                    .to_string(),
                approximate_area_sqft: 32.0,
            },
            SuggestedZone {
                label: "Seating Alcove".to_string(),
                zone_type: "seating".to_string(),
                rationale: "The sheltered corner west of the second London Plane is \
                    protected from prevailing winds and receives dappled light — ideal \
                    for a built-in bench with integrated planting."
                    .to_string(),
                approximate_area_sqft: 12.0,
            },
        ],
        site_observations: vec![
            SiteObservation {
                observation: "The two London Planes create a natural shade corridor \
                    along the north edge of the frontage, ideal for shade-loving \
                    understory planting such as Heuchera or Japanese Forest Grass."
                    .to_string(),
            },
            SiteObservation {
                observation: "The grade change near the curb (visible as elevation \
                    variation in the plan view) suggests a potential rain garden \
                    opportunity that would handle stormwater runoff from the sidewalk."
                    .to_string(),
            },
            SiteObservation {
                observation: "High pedestrian traffic at this intersection means any \
                    ground-level planting must tolerate occasional foot traffic. \
                    Dymondia or Carex pansa would be appropriate groundcovers."
                    .to_string(),
            },
        ],
    }
}
