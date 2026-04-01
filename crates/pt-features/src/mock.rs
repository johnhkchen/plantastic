//! Mock implementation of [`FeatureClassifier`] for tests.

use async_trait::async_trait;

use crate::baml_client::types::ClassifiedFeature;
use crate::classifier::FeatureClassifier;
use crate::error::ClassificationError;

/// Returns deterministic classifications based on geometric heuristics.
///
/// Same input always produces the same output. Classifications are plausible
/// for SF Bay Area urban environments.
#[derive(Debug)]
pub struct MockFeatureClassifier;

#[async_trait]
impl FeatureClassifier for MockFeatureClassifier {
    async fn classify(
        &self,
        candidates: &[pt_scan::FeatureCandidate],
        _address: &str,
        _climate_zone: &str,
    ) -> Result<Vec<ClassifiedFeature>, ClassificationError> {
        Ok(candidates.iter().map(classify_by_geometry).collect())
    }
}

/// Always returns a classification error. Use for testing error-handling paths.
#[derive(Debug)]
pub struct MockFailingClassifier;

#[async_trait]
impl FeatureClassifier for MockFailingClassifier {
    async fn classify(
        &self,
        _candidates: &[pt_scan::FeatureCandidate],
        _address: &str,
        _climate_zone: &str,
    ) -> Result<Vec<ClassifiedFeature>, ClassificationError> {
        Err(ClassificationError::Classification(
            "mock LLM failure: rate limit exceeded".to_string(),
        ))
    }
}

fn classify_by_geometry(c: &pt_scan::FeatureCandidate) -> ClassifiedFeature {
    let is_tall = c.height_ft > 8.0;
    let is_very_tall = c.height_ft > 15.0;
    let is_columnar = c.vertical_profile == "columnar";
    let is_flat = c.vertical_profile == "flat";
    let is_brown = c.dominant_color == "brown";
    let is_green = c.dominant_color == "green";
    let is_gray = c.dominant_color == "gray";
    let is_narrow = c.spread_ft < 2.0;
    let is_short = c.height_ft < 3.0;

    #[allow(clippy::cast_possible_truncation)]
    let cluster_id = c.cluster_id as i64;

    if is_tall && is_columnar && is_narrow && is_gray {
        return ClassifiedFeature {
            cluster_id,
            label: "Utility Pole".to_string(),
            category: "utility".to_string(),
            species: None,
            confidence: 0.75,
            reasoning: format!(
                "{:.1} ft tall columnar structure with {:.1} ft spread and gray color \
                 is consistent with a metal or wooden utility pole.",
                c.height_ft, c.spread_ft,
            ),
            landscape_notes: "Utility easement — maintain clearance for service access. \
                Consider screening with tall shrubs if not blocking sightlines."
                .to_string(),
        };
    }

    if is_tall && (is_columnar || c.vertical_profile == "spreading") && (is_brown || is_green) {
        let (label, species, confidence) = if is_very_tall && c.spread_ft > 5.0 {
            (
                "London Plane Tree".to_string(),
                Some("Platanus × acerifolia".to_string()),
                0.85,
            )
        } else if is_very_tall {
            (
                "Street Tree Trunk".to_string(),
                Some("Lophostemon confertus".to_string()),
                0.72,
            )
        } else {
            ("Young Street Tree".to_string(), None, 0.60)
        };

        return ClassifiedFeature {
            cluster_id,
            label,
            category: "tree".to_string(),
            species,
            confidence,
            reasoning: format!(
                "{:.1} ft height with {:.1} ft spread, {} profile and {} color. \
                 {} points at {:.0} pts/m³ density indicates solid trunk structure \
                 consistent with a mature street tree.",
                c.height_ft,
                c.spread_ft,
                c.vertical_profile,
                c.dominant_color,
                c.point_count,
                c.density,
            ),
            landscape_notes: "Root zone requires setback for adjacent hardscape. \
                Provides shade coverage for south- and west-facing areas."
                .to_string(),
        };
    }

    if is_short && is_flat {
        return ClassifiedFeature {
            cluster_id,
            label: if is_gray {
                "Concrete Curb".to_string()
            } else if is_brown {
                "Brick Pathway".to_string()
            } else {
                "Hardscape Element".to_string()
            },
            category: "hardscape".to_string(),
            species: None,
            confidence: 0.65,
            reasoning: format!(
                "{:.1} ft height with flat profile and {} color suggests \
                 a low hardscape feature such as a curb, pathway edge, or planter wall.",
                c.height_ft, c.dominant_color,
            ),
            landscape_notes: "Existing hardscape — assess condition before \
                incorporating into new design. May need re-grading for drainage."
                .to_string(),
        };
    }

    ClassifiedFeature {
        cluster_id,
        label: "Urban Structure".to_string(),
        category: "structure".to_string(),
        species: None,
        confidence: 0.45,
        reasoning: format!(
            "{:.1} ft height, {:.1} ft spread, {} profile, {} color — \
             insufficient geometric cues for confident classification. \
             Could be a bollard, planter, bench, or other urban furniture.",
            c.height_ft, c.spread_ft, c.vertical_profile, c.dominant_color,
        ),
        landscape_notes: "On-site verification recommended before incorporating \
            into landscape design."
            .to_string(),
    }
}
