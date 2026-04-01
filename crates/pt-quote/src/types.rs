//! Output types for the quote engine.

use pt_materials::{MaterialId, Unit};
use pt_project::{TierLevel, ZoneId};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A computed quote for a single tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quote {
    pub tier: TierLevel,
    pub line_items: Vec<LineItem>,
    pub subtotal: Decimal,
    pub tax: Option<Decimal>,
    pub total: Decimal,
}

/// A single line item in a quote.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineItem {
    pub zone_id: ZoneId,
    pub zone_label: Option<String>,
    pub material_id: MaterialId,
    pub material_name: String,
    pub quantity: Decimal,
    pub unit: Unit,
    pub unit_price: Decimal,
    pub line_total: Decimal,
}
