//! Quote computation engine.
//!
//! Takes zone geometries, a tier's material assignments, and a material catalog,
//! then produces a [`Quote`] with line items, subtotal, and total.
//! Pure computation — no I/O, no async.

use pt_geo::{area::area_sqft, perimeter::perimeter_ft, volume::volume_cuyd};
use pt_materials::{Material, MaterialId, Unit};
use pt_project::{AssignmentOverrides, MaterialAssignment, Tier, Zone};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

use crate::error::QuoteError;
use crate::types::{LineItem, Quote};

/// Compute a quote for a single tier.
///
/// Walks each material assignment in the tier, computes quantity from zone
/// geometry, multiplies by the material's unit price (or override), and
/// produces a complete [`Quote`] with line items and totals.
///
/// Zones that have no assignments in this tier are silently skipped.
///
/// # Errors
/// Returns [`QuoteError::MaterialNotFound`] if an assignment references a material
/// not in the catalog, or [`QuoteError::MissingDepth`] if a cu_yd material has no depth.
pub fn compute_quote(
    zones: &[Zone],
    tier: &Tier,
    materials: &[Material],
    tax: Option<Decimal>,
) -> Result<Quote, QuoteError> {
    let mut line_items = Vec::new();

    for assignment in &tier.assignments {
        let zone = match zones.iter().find(|z| z.id == assignment.zone_id) {
            Some(z) => z,
            None => continue, // zone not in project, skip defensively
        };

        let material = find_material(assignment.material_id, materials).ok_or(
            QuoteError::MaterialNotFound {
                material_id: assignment.material_id,
                zone_id: assignment.zone_id,
            },
        )?;

        let quantity = compute_quantity(zone, material, assignment.overrides.as_ref())?;
        let unit_price = effective_price(material, assignment);
        let line_total = round_currency(quantity * unit_price);
        let display_quantity = round_quantity(quantity);

        line_items.push(LineItem {
            zone_id: zone.id,
            zone_label: zone.label.clone(),
            material_id: material.id,
            material_name: material.name.clone(),
            quantity: display_quantity,
            unit: material.unit,
            unit_price,
            line_total,
        });
    }

    let subtotal: Decimal = line_items.iter().map(|li| li.line_total).sum();
    let tax_amount = tax.unwrap_or(Decimal::ZERO);
    let total = subtotal + tax_amount;

    Ok(Quote {
        tier: tier.level,
        line_items,
        subtotal,
        tax,
        total,
    })
}

fn find_material(id: MaterialId, materials: &[Material]) -> Option<&Material> {
    materials.iter().find(|m| m.id == id)
}

fn compute_quantity(
    zone: &Zone,
    material: &Material,
    overrides: Option<&AssignmentOverrides>,
) -> Result<Decimal, QuoteError> {
    let raw = match material.unit {
        Unit::SqFt => area_sqft(&zone.geometry),
        Unit::LinearFt => perimeter_ft(&zone.geometry),
        Unit::Each => 1.0,
        Unit::CuYd => {
            let depth_inches = overrides
                .and_then(|o| o.depth_override_inches)
                .or(material.depth_inches)
                .ok_or_else(|| QuoteError::MissingDepth {
                    material_id: material.id,
                    material_name: material.name.clone(),
                })?;
            let area = area_sqft(&zone.geometry);
            let depth_ft = depth_inches / 12.0;
            volume_cuyd(area, depth_ft)
        }
    };

    Ok(Decimal::from_f64(raw).unwrap_or(Decimal::ZERO))
}

fn effective_price(material: &Material, assignment: &MaterialAssignment) -> Decimal {
    assignment
        .overrides
        .as_ref()
        .and_then(|o| o.price_override)
        .unwrap_or(material.price_per_unit)
}

fn round_currency(d: Decimal) -> Decimal {
    d.round_dp(2)
}

fn round_quantity(d: Decimal) -> Decimal {
    d.round_dp(4)
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::polygon;
    use pt_materials::{ExtrusionBehavior, MaterialCategory};
    use pt_project::{TierLevel, ZoneId, ZoneType};
    use pt_test_utils::timed;
    use std::str::FromStr;

    /// Convenience: parse a Decimal from a string literal.
    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn make_zone(w: f64, h: f64, zone_type: ZoneType, label: Option<&str>) -> Zone {
        Zone {
            id: ZoneId::new(),
            geometry: polygon![
                (x: 0.0, y: 0.0),
                (x: w,   y: 0.0),
                (x: w,   y: h),
                (x: 0.0, y: h),
            ],
            zone_type,
            label: label.map(String::from),
        }
    }

    fn make_sqft_material(name: &str, price: &str) -> Material {
        Material::builder(name, MaterialCategory::Hardscape)
            .unit(Unit::SqFt)
            .price_per_unit(Decimal::from_str(price).unwrap())
            .extrusion(ExtrusionBehavior::SitsOnTop { height_inches: 1.0 })
            .build()
    }

    fn make_cuyd_material(name: &str, price: &str, depth_inches: f64) -> Material {
        Material::builder(name, MaterialCategory::Softscape)
            .unit(Unit::CuYd)
            .price_per_unit(Decimal::from_str(price).unwrap())
            .depth_inches(depth_inches)
            .extrusion(ExtrusionBehavior::Fills { flush: true })
            .build()
    }

    fn make_linearft_material(name: &str, price: &str) -> Material {
        Material::builder(name, MaterialCategory::Edging)
            .unit(Unit::LinearFt)
            .price_per_unit(Decimal::from_str(price).unwrap())
            .extrusion(ExtrusionBehavior::SitsOnTop { height_inches: 4.0 })
            .build()
    }

    fn make_each_material(name: &str, price: &str) -> Material {
        Material::builder(name, MaterialCategory::Hardscape)
            .unit(Unit::Each)
            .price_per_unit(Decimal::from_str(price).unwrap())
            .build()
    }

    fn assign(zone_id: ZoneId, material_id: MaterialId) -> MaterialAssignment {
        MaterialAssignment {
            zone_id,
            material_id,
            overrides: None,
        }
    }

    fn assign_with_overrides(
        zone_id: ZoneId,
        material_id: MaterialId,
        overrides: AssignmentOverrides,
    ) -> MaterialAssignment {
        MaterialAssignment {
            zone_id,
            material_id,
            overrides: Some(overrides),
        }
    }

    #[test]
    fn sqft_10x10_patio_at_5_per_sqft() {
        timed(|| {
            let zone = make_zone(10.0, 10.0, ZoneType::Patio, Some("Back patio"));
            let mat = make_sqft_material("Pavers", "5.00");
            let tier = Tier {
                level: TierLevel::Good,
                assignments: vec![assign(zone.id, mat.id)],
            };

            let quote = compute_quote(&[zone], &tier, &[mat], None).unwrap();

            assert_eq!(quote.line_items.len(), 1);
            // 10 * 10 = 100 sq_ft * $5.00 = $500.00
            assert_eq!(quote.line_items[0].line_total, d("500.00"));
            assert_eq!(quote.subtotal, d("500.00"));
            assert_eq!(quote.total, d("500.00"));
        });
    }

    #[test]
    fn cuyd_material_with_depth() {
        timed(|| {
            let zone = make_zone(10.0, 10.0, ZoneType::Bed, Some("Garden bed"));
            let mat = make_cuyd_material("Mulch", "40.00", 4.0);
            let tier = Tier {
                level: TierLevel::Good,
                assignments: vec![assign(zone.id, mat.id)],
            };

            let quote = compute_quote(&[zone], &tier, &[mat], None).unwrap();

            // 100 sq_ft * (4/12) ft / 27 = 100 * 0.3333... / 27 = 1.2345679... cu_yd
            // 1.2345679... * $40.00 = $49.38 (rounded)
            assert_eq!(quote.line_items[0].line_total, d("49.38"));
        });
    }

    #[test]
    fn linearft_edging() {
        timed(|| {
            let zone = make_zone(10.0, 10.0, ZoneType::Edging, Some("Border"));
            let mat = make_linearft_material("Steel Edge", "2.50");
            let tier = Tier {
                level: TierLevel::Good,
                assignments: vec![assign(zone.id, mat.id)],
            };

            let quote = compute_quote(&[zone], &tier, &[mat], None).unwrap();

            // Perimeter of 10x10 = 40 linear_ft * $2.50 = $100.00
            assert_eq!(quote.line_items[0].line_total, d("100.00"));
        });
    }

    #[test]
    fn each_material() {
        timed(|| {
            let zone = make_zone(5.0, 5.0, ZoneType::Patio, Some("Fountain pad"));
            let mat = make_each_material("Fountain", "150.00");
            let tier = Tier {
                level: TierLevel::Good,
                assignments: vec![assign(zone.id, mat.id)],
            };

            let quote = compute_quote(&[zone], &tier, &[mat], None).unwrap();

            // 1 × $150.00 = $150.00
            assert_eq!(quote.line_items[0].quantity, d("1.0"));
            assert_eq!(quote.line_items[0].line_total, d("150.00"));
        });
    }

    #[test]
    fn zone_with_no_assignment_skipped() {
        timed(|| {
            let z1 = make_zone(10.0, 10.0, ZoneType::Patio, Some("Patio"));
            let z2 = make_zone(5.0, 5.0, ZoneType::Bed, Some("Bed"));
            let mat = make_sqft_material("Pavers", "5.00");
            let tier = Tier {
                level: TierLevel::Good,
                assignments: vec![assign(z1.id, mat.id)], // only z1 assigned
            };

            let quote = compute_quote(&[z1, z2], &tier, &[mat], None).unwrap();

            assert_eq!(quote.line_items.len(), 1);
            assert_eq!(quote.line_items[0].zone_label, Some("Patio".to_string()));
        });
    }

    #[test]
    fn multiple_materials_per_zone() {
        timed(|| {
            let zone = make_zone(10.0, 10.0, ZoneType::Patio, Some("Patio"));
            let gravel = make_cuyd_material("Gravel Base", "30.00", 3.0);
            let pavers = make_sqft_material("Pavers", "8.00");
            let tier = Tier {
                level: TierLevel::Good,
                assignments: vec![assign(zone.id, gravel.id), assign(zone.id, pavers.id)],
            };

            let quote = compute_quote(&[zone], &tier, &[gravel, pavers], None).unwrap();

            assert_eq!(quote.line_items.len(), 2);

            // Gravel: 100 sq_ft * (3/12) / 27 = 0.9259... cu_yd * $30 = $27.78
            assert_eq!(quote.line_items[0].line_total, d("27.78"));
            // Pavers: 100 sq_ft * $8.00 = $800.00
            assert_eq!(quote.line_items[1].line_total, d("800.00"));
            // Subtotal: $27.78 + $800.00 = $827.78
            assert_eq!(quote.subtotal, d("827.78"));
        });
    }

    #[test]
    fn override_price() {
        timed(|| {
            let zone = make_zone(10.0, 10.0, ZoneType::Patio, None);
            let mat = make_sqft_material("Pavers", "5.00");
            let tier = Tier {
                level: TierLevel::Good,
                assignments: vec![assign_with_overrides(
                    zone.id,
                    mat.id,
                    AssignmentOverrides {
                        price_override: Some(Decimal::from_str("7.00").unwrap()),
                        depth_override_inches: None,
                    },
                )],
            };

            let quote = compute_quote(&[zone], &tier, &[mat], None).unwrap();

            // 100 sq_ft * $7.00 (override) = $700.00
            assert_eq!(quote.line_items[0].unit_price, d("7.00"));
            assert_eq!(quote.line_items[0].line_total, d("700.00"));
        });
    }

    #[test]
    fn override_depth() {
        timed(|| {
            let zone = make_zone(10.0, 10.0, ZoneType::Bed, None);
            let mat = make_cuyd_material("Mulch", "40.00", 3.0);
            let tier = Tier {
                level: TierLevel::Good,
                assignments: vec![assign_with_overrides(
                    zone.id,
                    mat.id,
                    AssignmentOverrides {
                        price_override: None,
                        depth_override_inches: Some(6.0),
                    },
                )],
            };

            let quote = compute_quote(&[zone], &tier, &[mat], None).unwrap();

            // 100 sq_ft * (6/12) / 27 = 1.8518... cu_yd * $40 = $74.07
            assert_eq!(quote.line_items[0].line_total, d("74.07"));
        });
    }

    #[test]
    fn missing_material_returns_error() {
        timed(|| {
            let zone = make_zone(10.0, 10.0, ZoneType::Patio, None);
            let zone_id = zone.id;
            let fake_id = MaterialId::new();
            let tier = Tier {
                level: TierLevel::Good,
                assignments: vec![assign(zone_id, fake_id)],
            };

            let err = compute_quote(&[zone], &tier, &[], None).unwrap_err();
            assert_eq!(
                err,
                QuoteError::MaterialNotFound {
                    material_id: fake_id,
                    zone_id,
                }
            );
        });
    }

    #[test]
    fn missing_depth_for_cuyd_returns_error() {
        timed(|| {
            let zone = make_zone(10.0, 10.0, ZoneType::Bed, None);
            // Build a cu_yd material with NO depth
            let mat = Material::builder("No-depth Mulch", MaterialCategory::Softscape)
                .unit(Unit::CuYd)
                .price_per_unit(Decimal::from_str("40.00").unwrap())
                .extrusion(ExtrusionBehavior::Fills { flush: true })
                .build();
            let tier = Tier {
                level: TierLevel::Good,
                assignments: vec![assign(zone.id, mat.id)],
            };

            let err = compute_quote(&[zone], &tier, std::slice::from_ref(&mat), None).unwrap_err();
            assert_eq!(
                err,
                QuoteError::MissingDepth {
                    material_id: mat.id,
                    material_name: "No-depth Mulch".to_string(),
                }
            );
        });
    }

    #[test]
    fn empty_tier_produces_empty_quote() {
        timed(|| {
            let zone = make_zone(10.0, 10.0, ZoneType::Patio, None);
            let tier = Tier {
                level: TierLevel::Good,
                assignments: vec![],
            };

            let quote = compute_quote(&[zone], &tier, &[], None).unwrap();

            assert!(quote.line_items.is_empty());
            assert_eq!(quote.subtotal, Decimal::ZERO);
            assert_eq!(quote.total, Decimal::ZERO);
        });
    }

    #[test]
    fn tax_included_in_total() {
        timed(|| {
            let zone = make_zone(10.0, 10.0, ZoneType::Patio, None);
            let mat = make_sqft_material("Pavers", "5.00");
            let tier = Tier {
                level: TierLevel::Good,
                assignments: vec![assign(zone.id, mat.id)],
            };
            let tax = Some(Decimal::from_str("42.50").unwrap());

            let quote = compute_quote(&[zone], &tier, &[mat], tax).unwrap();

            assert_eq!(quote.subtotal, d("500.00"));
            assert_eq!(quote.tax, Some(d("42.50")));
            // $500.00 + $42.50 = $542.50
            assert_eq!(quote.total, d("542.50"));
        });
    }
}
