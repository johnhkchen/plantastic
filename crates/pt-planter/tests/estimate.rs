//! Integration tests for planter estimation.

use pt_planter::{
    compute_style, MockFailingEstimator, MockPlanterEstimator, PlanterEstimator, PlanterInput,
};
use pt_test_utils::timed;
use rust_decimal::Decimal;

fn powell_market_input() -> PlanterInput {
    PlanterInput {
        gap_width_ft: 8.0,
        gap_length_ft: 2.3,
        area_sqft: 18.4,
        adjacent_features: vec![
            "London Plane Tree".to_string(),
            "London Plane Tree".to_string(),
        ],
        sun_hours: Some(4),
        climate_zone: "USDA 10b / Sunset 17 — Mediterranean maritime".to_string(),
        address: "Powell & Market Streets, San Francisco, CA".to_string(),
    }
}

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

#[test]
fn mock_returns_three_styles() {
    timed(|| {
        let result = block_on(async {
            let estimator = MockPlanterEstimator;
            estimator.estimate(&powell_market_input()).await.unwrap()
        });

        assert_eq!(result.styles.len(), 3, "expected 3 styles");

        for style in &result.styles {
            assert!(
                style.plant_selections.len() >= 2,
                "style '{}' has {} plants, expected >=2",
                style.style_name,
                style.plant_selections.len()
            );
            assert!(style.soil_depth_inches > 0.0, "soil depth must be positive");
            assert!(!style.style_name.is_empty(), "style name must not be empty");
            assert!(
                !style.design_rationale.is_empty(),
                "design rationale must not be empty"
            );
        }

        // Style names are distinct
        let names: Vec<&str> = result
            .styles
            .iter()
            .map(|s| s.style_name.as_str())
            .collect();
        assert_ne!(names[0], names[1]);
        assert_ne!(names[1], names[2]);
    });
}

#[test]
fn mock_computed_costs_match_arithmetic() {
    timed(|| {
        let result = block_on(async {
            let estimator = MockPlanterEstimator;
            estimator.estimate(&powell_market_input()).await.unwrap()
        });
        let area = 18.4;

        for style in &result.styles {
            let computed = compute_style(style, area);

            // Verify total = sum of plant costs + soil cost
            let plant_total: Decimal = computed.plantings.iter().map(|p| p.plant_cost).sum();
            let expected_total = plant_total + computed.soil_cost;
            assert_eq!(
                computed.total_cost, expected_total,
                "style '{}': total {} != plant {} + soil {}",
                computed.style_name, computed.total_cost, plant_total, computed.soil_cost
            );

            // Verify each planting cost = count * unit_price
            for planting in &computed.plantings {
                let expected_cost = Decimal::from(planting.plant_count) * planting.unit_price;
                assert_eq!(
                    planting.plant_cost,
                    expected_cost,
                    "plant '{}': cost {} != {} * {}",
                    planting.common_name,
                    planting.plant_cost,
                    planting.plant_count,
                    planting.unit_price,
                );
            }

            // Total cost must be positive
            assert!(
                computed.total_cost > Decimal::ZERO,
                "style '{}' total cost should be positive",
                computed.style_name,
            );
        }

        // Style 1 (budget) should cost less than Style 3 (premium)
        let cost_1 = compute_style(&result.styles[0], area).total_cost;
        let cost_3 = compute_style(&result.styles[2], area).total_cost;
        assert!(
            cost_1 < cost_3,
            "budget style (${cost_1}) should cost less than premium (${cost_3})"
        );
    });
}

#[test]
fn failing_estimator_returns_error() {
    timed(|| {
        let result = block_on(async {
            let estimator = MockFailingEstimator;
            estimator.estimate(&powell_market_input()).await
        });
        assert!(result.is_err(), "failing estimator should return error");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("rate limit"),
            "error should mention rate limit: {err}"
        );
    });
}

#[test]
fn empty_adjacent_features_works() {
    timed(|| {
        let mut input = powell_market_input();
        input.adjacent_features = vec![];
        let result = block_on(async {
            let estimator = MockPlanterEstimator;
            estimator.estimate(&input).await
        });
        assert!(result.is_ok(), "empty adjacent features should not fail");
    });
}
