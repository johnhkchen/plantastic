//! Mock implementation of [`PlanterEstimator`] for tests.

use async_trait::async_trait;

use crate::baml_client::types::{PlantSelection, PlanterEstimate, PlanterStyle};
use crate::error::PlanterError;
use crate::estimator::{PlanterEstimator, PlanterInput};

/// Returns deterministic planter estimates with real SF Bay Area plants.
///
/// Three styles ranging from budget-friendly to premium, appropriate for
/// the Powell & Market gap scenario (partial shade, high foot traffic,
/// USDA 10b / Sunset 17).
#[derive(Debug)]
pub struct MockPlanterEstimator;

#[async_trait]
impl PlanterEstimator for MockPlanterEstimator {
    async fn estimate(&self, _input: &PlanterInput) -> Result<PlanterEstimate, PlanterError> {
        Ok(powell_market_fixture())
    }
}

/// Always returns an estimation error. Use for testing error-handling paths.
#[derive(Debug)]
pub struct MockFailingEstimator;

#[async_trait]
impl PlanterEstimator for MockFailingEstimator {
    async fn estimate(&self, _input: &PlanterInput) -> Result<PlanterEstimate, PlanterError> {
        Err(PlanterError::Estimation(
            "mock LLM failure: rate limit exceeded".to_string(),
        ))
    }
}

/// Hand-crafted fixture for Powell & Market Streets gap.
///
/// Two London Plane trunks, ~18.4 sqft gap, partial shade, high foot traffic.
/// All plants are real species appropriate for SF USDA 10b / Sunset 17.
fn powell_market_fixture() -> PlanterEstimate {
    PlanterEstimate {
        styles: vec![
            // Style 1: Budget — drought-tolerant groundcover
            PlanterStyle {
                style_name: "Drought-Tolerant Groundcover".to_string(),
                description: "Low-maintenance carpet of silver-green foliage that \
                    handles foot traffic and requires minimal irrigation once established."
                    .to_string(),
                plant_selections: vec![
                    PlantSelection {
                        common_name: "Dymondia".to_string(),
                        botanical_name: "Dymondia margaretae".to_string(),
                        spacing_inches: 12.0,
                        why_this_plant: "Silver mat-forming groundcover that tolerates \
                            light foot traffic and thrives in partial shade between trunks."
                            .to_string(),
                    },
                    PlantSelection {
                        common_name: "Carex pansa".to_string(),
                        botanical_name: "Carex pansa".to_string(),
                        spacing_inches: 12.0,
                        why_this_plant: "Native California meadow sedge that fills gaps \
                            with fine-textured green — no mowing, minimal water."
                            .to_string(),
                    },
                ],
                soil_depth_inches: 4.0,
                design_rationale: "Maximum drought tolerance with minimal maintenance — \
                    ideal for a high-traffic streetscape where irrigation access is limited."
                    .to_string(),
            },
            // Style 2: Mid-range — mixed perennial bed
            PlanterStyle {
                style_name: "Mixed Perennial Bed".to_string(),
                description: "A layered planting of colorful perennials and ornamental \
                    grasses that provides year-round interest with moderate maintenance."
                    .to_string(),
                plant_selections: vec![
                    PlantSelection {
                        common_name: "Salvia 'Hot Lips'".to_string(),
                        botanical_name: "Salvia microphylla 'Hot Lips'".to_string(),
                        spacing_inches: 12.0,
                        why_this_plant: "Bicolor red-white flowers attract pollinators \
                            and provide vertical interest between the tree trunks."
                            .to_string(),
                    },
                    PlantSelection {
                        common_name: "Coral Bells".to_string(),
                        botanical_name: "Heuchera 'Palace Purple'".to_string(),
                        spacing_inches: 10.0,
                        why_this_plant: "Deep purple foliage adds contrast at ground level \
                            and tolerates the partial shade cast by adjacent buildings."
                            .to_string(),
                    },
                    PlantSelection {
                        common_name: "Blue Fescue".to_string(),
                        botanical_name: "Festuca glauca 'Elijah Blue'".to_string(),
                        spacing_inches: 8.0,
                        why_this_plant: "Compact blue-grey tufts provide texture and \
                            movement — drought-tolerant once established."
                            .to_string(),
                    },
                ],
                soil_depth_inches: 8.0,
                design_rationale: "Three-layer planting with seasonal color, textural \
                    contrast, and moderate water needs — a step up from basic groundcover \
                    that reads as intentional landscape design."
                    .to_string(),
            },
            // Style 3: Premium — statement planting
            PlanterStyle {
                style_name: "Urban Woodland Edge".to_string(),
                description: "A refined composition centered on a specimen Japanese Maple \
                    with cascading grasses and a tight groundcover matrix."
                    .to_string(),
                plant_selections: vec![
                    PlantSelection {
                        common_name: "Japanese Forest Grass".to_string(),
                        botanical_name: "Hakonechloa macra 'Aureola'".to_string(),
                        spacing_inches: 8.0,
                        why_this_plant: "Golden cascading foliage catches light in partial \
                            shade and provides year-round movement and texture."
                            .to_string(),
                    },
                    PlantSelection {
                        common_name: "Coral Bells".to_string(),
                        botanical_name: "Heuchera 'Obsidian'".to_string(),
                        spacing_inches: 8.0,
                        why_this_plant: "Near-black foliage provides dramatic contrast \
                            against the golden Hakonechloa — a designer pairing."
                            .to_string(),
                    },
                    PlantSelection {
                        common_name: "Mondo Grass".to_string(),
                        botanical_name: "Ophiopogon japonicus 'Nana'".to_string(),
                        spacing_inches: 4.0,
                        why_this_plant: "Dense dark-green groundcover fills every gap \
                            and creates a clean, manicured edge along the sidewalk."
                            .to_string(),
                    },
                ],
                soil_depth_inches: 10.0,
                design_rationale: "Three-texture layering with dramatic color contrast. \
                    Dense, tight spacing creates a polished, high-impact composition \
                    worthy of a landmark intersection."
                    .to_string(),
            },
        ],
    }
}
