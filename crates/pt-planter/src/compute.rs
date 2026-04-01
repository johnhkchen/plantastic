//! Code-computed quantities and costs for planter styles.
//!
//! The LLM picks plants (what + why). This module computes quantities
//! and costs (how many + how much) from the LLM's spacing and depth values.

use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use serde::Serialize;

use crate::baml_client::types::PlanterStyle;

// ── Default pricing ─────────────────────────────────────────────

/// Grasses: $15/gal container.
const GRASS_PRICE: &str = "15.00";
/// Annuals: $8/4" pot.
const ANNUAL_PRICE: &str = "8.00";
/// Succulents: $12/gal container.
const SUCCULENT_PRICE: &str = "12.00";
/// Default plant price when category is unknown.
const DEFAULT_PRICE: &str = "12.00";
/// Soil: $45/cubic yard.
const SOIL_PRICE_PER_CUYD: &str = "45.00";

/// Computed quantities for a single plant selection within a style.
#[derive(Debug, Clone, Serialize)]
pub struct ComputedPlanting {
    pub common_name: String,
    pub botanical_name: String,
    pub plant_count: u32,
    pub unit_price: Decimal,
    pub plant_cost: Decimal,
}

/// Computed quantities and costs for an entire planter style.
#[derive(Debug, Clone, Serialize)]
pub struct ComputedStyle {
    pub style_name: String,
    pub plantings: Vec<ComputedPlanting>,
    pub soil_volume_cuyd: Decimal,
    pub soil_cost: Decimal,
    pub total_cost: Decimal,
}

/// Compute quantities and costs for a planter style given the gap area.
///
/// For each plant selection:
///   `plant_count = floor(area_sqft / (spacing_inches / 12)^2)`
///
/// Soil volume:
///   `soil_volume_cuyd = area_sqft * depth_inches / (12 * 27)`
///
/// The area is split evenly among plant selections for count computation.
pub fn compute_style(style: &PlanterStyle, area_sqft: f64) -> ComputedStyle {
    let area = Decimal::from_f64(area_sqft).unwrap_or(Decimal::ZERO);
    let soil_price = Decimal::from_str(SOIL_PRICE_PER_CUYD).unwrap();
    let num_plants = style.plant_selections.len().max(1);
    let area_per_plant = area / Decimal::from(num_plants);

    let plantings: Vec<ComputedPlanting> = style
        .plant_selections
        .iter()
        .map(|sel| {
            let unit_price = infer_price_category(&sel.common_name);
            let count = plant_count(area_per_plant, sel.spacing_inches);
            let cost = Decimal::from(count) * unit_price;

            ComputedPlanting {
                common_name: sel.common_name.clone(),
                botanical_name: sel.botanical_name.clone(),
                plant_count: count,
                unit_price,
                plant_cost: round_currency(cost),
            }
        })
        .collect();

    let soil_vol = soil_volume_cuyd(area, style.soil_depth_inches);
    let soil_cost = round_currency(soil_vol * soil_price);

    let total = plantings.iter().map(|p| p.plant_cost).sum::<Decimal>() + soil_cost;

    ComputedStyle {
        style_name: style.style_name.clone(),
        plantings,
        soil_volume_cuyd: round_quantity(soil_vol),
        soil_cost,
        total_cost: round_currency(total),
    }
}

/// Compute plant count: `floor(area_sqft / (spacing_inches / 12)^2)`.
fn plant_count(area_sqft: Decimal, spacing_inches: f64) -> u32 {
    if spacing_inches <= 0.0 {
        return 0;
    }
    let spacing_ft = spacing_inches / 12.0;
    let spacing_area = Decimal::from_f64(spacing_ft * spacing_ft).unwrap_or(Decimal::ONE);
    if spacing_area.is_zero() {
        return 0;
    }
    let count = area_sqft / spacing_area;
    count.floor().to_u32().unwrap_or(0)
}

/// Compute soil volume: `area_sqft * depth_inches / (12 * 27)`.
fn soil_volume_cuyd(area_sqft: Decimal, depth_inches: f64) -> Decimal {
    let depth = Decimal::from_f64(depth_inches).unwrap_or(Decimal::ZERO);
    // 12 inches/ft * 27 ft³/yd³ = 324
    let divisor = Decimal::from(324);
    area_sqft * depth / divisor
}

/// Infer pricing category from common plant name.
///
/// Simple keyword matching — not worth an LLM call.
pub fn infer_price_category(common_name: &str) -> Decimal {
    let lower = common_name.to_lowercase();

    // Grasses
    if lower.contains("grass")
        || lower.contains("sedge")
        || lower.contains("carex")
        || lower.contains("fescue")
        || lower.contains("festuca")
        || lower.contains("hakonechloa")
        || lower.contains("mondo")
    {
        return Decimal::from_str(GRASS_PRICE).unwrap();
    }

    // Annuals
    if lower.contains("annual")
        || lower.contains("petunia")
        || lower.contains("marigold")
        || lower.contains("impatiens")
        || lower.contains("begonia")
    {
        return Decimal::from_str(ANNUAL_PRICE).unwrap();
    }

    // Succulents
    if lower.contains("succulent")
        || lower.contains("sedum")
        || lower.contains("echeveria")
        || lower.contains("agave")
        || lower.contains("aloe")
        || lower.contains("dymondia")
    {
        return Decimal::from_str(SUCCULENT_PRICE).unwrap();
    }

    Decimal::from_str(DEFAULT_PRICE).unwrap()
}

fn round_currency(value: Decimal) -> Decimal {
    value.round_dp(2)
}

fn round_quantity(value: Decimal) -> Decimal {
    value.round_dp(3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::baml_client::types::PlantSelection;
    use pt_test_utils::timed;

    fn make_selection(name: &str, botanical: &str, spacing: f64) -> PlantSelection {
        PlantSelection {
            common_name: name.to_string(),
            botanical_name: botanical.to_string(),
            spacing_inches: spacing,
            why_this_plant: "test".to_string(),
        }
    }

    fn make_style(name: &str, selections: Vec<PlantSelection>, depth: f64) -> PlanterStyle {
        PlanterStyle {
            style_name: name.to_string(),
            description: "test style".to_string(),
            plant_selections: selections,
            soil_depth_inches: depth,
            design_rationale: "test".to_string(),
        }
    }

    #[test]
    fn test_plant_count_known_values() {
        timed(|| {
            // 16 sqft, 6" spacing → spacing_ft = 0.5, area_per_plant = 0.25 sqft
            // count = floor(16 / 0.25) = 64
            let area = Decimal::from(16);
            assert_eq!(plant_count(area, 6.0), 64);

            // 100 sqft, 12" spacing → spacing_ft = 1.0, area = 1.0
            // count = floor(100 / 1.0) = 100
            let area = Decimal::from(100);
            assert_eq!(plant_count(area, 12.0), 100);

            // 18.4 sqft, 8" spacing → spacing_ft = 0.667, area = 0.444
            // count = floor(18.4 / 0.444) = floor(41.4) = 41
            let area = Decimal::from_f64(18.4).unwrap();
            assert_eq!(plant_count(area, 8.0), 41);
        });
    }

    #[test]
    fn test_soil_volume_known_values() {
        timed(|| {
            // 16 sqft, 8" depth → 16 * 8 / 324 = 128 / 324 = 0.395 cuyd
            let vol = soil_volume_cuyd(Decimal::from(16), 8.0);
            let expected = Decimal::from_str("0.395").unwrap();
            assert!(
                (vol - expected).abs() < Decimal::from_str("0.001").unwrap(),
                "soil volume: {vol}, expected ~{expected}"
            );

            // 100 sqft, 6" depth → 100 * 6 / 324 = 600 / 324 = 1.852 cuyd
            let vol = soil_volume_cuyd(Decimal::from(100), 6.0);
            let expected = Decimal::from_str("1.852").unwrap();
            assert!(
                (vol - expected).abs() < Decimal::from_str("0.001").unwrap(),
                "soil volume: {vol}, expected ~{expected}"
            );
        });
    }

    #[test]
    fn test_infer_price_category() {
        timed(|| {
            assert_eq!(
                infer_price_category("Blue Fescue"),
                Decimal::from_str("15.00").unwrap()
            );
            assert_eq!(
                infer_price_category("Carex pansa"),
                Decimal::from_str("15.00").unwrap()
            );
            assert_eq!(
                infer_price_category("Mondo Grass"),
                Decimal::from_str("15.00").unwrap()
            );
            assert_eq!(
                infer_price_category("Dymondia"),
                Decimal::from_str("12.00").unwrap()
            );
            assert_eq!(
                infer_price_category("Petunia"),
                Decimal::from_str("8.00").unwrap()
            );
            assert_eq!(
                infer_price_category("Salvia 'Hot Lips'"),
                Decimal::from_str("12.00").unwrap()
            );
        });
    }

    #[test]
    fn test_compute_style_two_plants() {
        timed(|| {
            // 18.4 sqft gap, 2 plants, 8" soil depth
            // Each plant gets 9.2 sqft
            //
            // Plant A: Carex pansa, 8" spacing (grass: $15)
            //   count = floor(9.2 / (8/12)^2) = floor(9.2 / 0.444) = floor(20.7) = 20
            //   cost = 20 * $15 = $300.00
            //
            // Plant B: Salvia, 12" spacing (default: $12)
            //   count = floor(9.2 / (12/12)^2) = floor(9.2 / 1.0) = 9
            //   cost = 9 * $12 = $108.00
            //
            // Soil: 18.4 * 8 / 324 = 147.2 / 324 = 0.454 cuyd
            //   soil_cost = 0.454 * $45 = $20.43 (rounded)
            //
            // Total = $300 + $108 + $20.43 = $428.43
            let style = make_style(
                "Test",
                vec![
                    make_selection("Carex pansa", "Carex pansa", 8.0),
                    make_selection("Salvia 'Hot Lips'", "Salvia microphylla 'Hot Lips'", 12.0),
                ],
                8.0,
            );

            let computed = compute_style(&style, 18.4);

            assert_eq!(computed.plantings.len(), 2);
            assert_eq!(computed.plantings[0].plant_count, 20);
            assert_eq!(
                computed.plantings[0].unit_price,
                Decimal::from_str("15.00").unwrap()
            );
            assert_eq!(
                computed.plantings[0].plant_cost,
                Decimal::from_str("300.00").unwrap()
            );

            assert_eq!(computed.plantings[1].plant_count, 9);
            assert_eq!(
                computed.plantings[1].unit_price,
                Decimal::from_str("12.00").unwrap()
            );
            assert_eq!(
                computed.plantings[1].plant_cost,
                Decimal::from_str("108.00").unwrap()
            );

            // Soil: 18.4 * 8 / 324 = 0.454
            let expected_soil_vol = Decimal::from_str("0.454").unwrap();
            assert!(
                (computed.soil_volume_cuyd - expected_soil_vol).abs()
                    < Decimal::from_str("0.001").unwrap(),
                "soil vol: {}, expected ~{}",
                computed.soil_volume_cuyd,
                expected_soil_vol
            );

            // Soil cost: 0.454... * 45 = 20.44 (from precise value)
            // Verify total is sum of parts
            let parts_total = computed
                .plantings
                .iter()
                .map(|p| p.plant_cost)
                .sum::<Decimal>()
                + computed.soil_cost;
            assert_eq!(computed.total_cost, parts_total);
        });
    }

    #[test]
    fn test_zero_spacing_returns_zero_count() {
        timed(|| {
            assert_eq!(plant_count(Decimal::from(100), 0.0), 0);
            assert_eq!(plant_count(Decimal::from(100), -1.0), 0);
        });
    }
}
