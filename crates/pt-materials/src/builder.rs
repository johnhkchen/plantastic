//! Builder pattern for constructing [`Material`] instances.

use rust_decimal::Decimal;

use crate::types::{ExtrusionBehavior, Material, MaterialCategory, MaterialId, Unit};

/// Builder for [`Material`] with sensible defaults.
///
/// Required fields (name, category) are set at construction. All other fields
/// have defaults and can be overridden via method chaining.
#[derive(Debug)]
pub struct MaterialBuilder {
    id: Option<MaterialId>,
    name: String,
    category: MaterialCategory,
    unit: Unit,
    price_per_unit: Decimal,
    depth_inches: Option<f64>,
    texture_ref: Option<String>,
    photo_ref: Option<String>,
    supplier_sku: Option<String>,
    extrusion: ExtrusionBehavior,
}

impl Material {
    /// Create a builder with the required fields.
    pub fn builder(name: impl Into<String>, category: MaterialCategory) -> MaterialBuilder {
        MaterialBuilder {
            id: None,
            name: name.into(),
            category,
            unit: Unit::SqFt,
            price_per_unit: Decimal::ZERO,
            depth_inches: None,
            texture_ref: None,
            photo_ref: None,
            supplier_sku: None,
            extrusion: ExtrusionBehavior::SitsOnTop { height_inches: 1.0 },
        }
    }
}

impl MaterialBuilder {
    pub fn id(mut self, id: MaterialId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn unit(mut self, unit: Unit) -> Self {
        self.unit = unit;
        self
    }

    pub fn price_per_unit(mut self, price: Decimal) -> Self {
        self.price_per_unit = price;
        self
    }

    pub fn depth_inches(mut self, depth: f64) -> Self {
        self.depth_inches = Some(depth);
        self
    }

    pub fn texture_ref(mut self, texture: impl Into<String>) -> Self {
        self.texture_ref = Some(texture.into());
        self
    }

    pub fn photo_ref(mut self, photo: impl Into<String>) -> Self {
        self.photo_ref = Some(photo.into());
        self
    }

    pub fn supplier_sku(mut self, sku: impl Into<String>) -> Self {
        self.supplier_sku = Some(sku.into());
        self
    }

    pub fn extrusion(mut self, behavior: ExtrusionBehavior) -> Self {
        self.extrusion = behavior;
        self
    }

    /// Consume the builder and produce a [`Material`].
    pub fn build(self) -> Material {
        Material {
            id: self.id.unwrap_or_default(),
            name: self.name,
            category: self.category,
            unit: self.unit,
            price_per_unit: self.price_per_unit,
            depth_inches: self.depth_inches,
            texture_ref: self.texture_ref,
            photo_ref: self.photo_ref,
            supplier_sku: self.supplier_sku,
            extrusion: self.extrusion,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn builder_required_fields_only() {
        let mat = Material::builder("Basic Mulch", MaterialCategory::Softscape).build();

        assert_eq!(mat.name, "Basic Mulch");
        assert_eq!(mat.category, MaterialCategory::Softscape);
        assert_eq!(mat.unit, Unit::SqFt);
        assert_eq!(mat.price_per_unit, Decimal::ZERO);
        assert_eq!(mat.depth_inches, None);
        assert_eq!(mat.texture_ref, None);
        assert_eq!(mat.photo_ref, None);
        assert_eq!(mat.supplier_sku, None);
        assert_eq!(
            mat.extrusion,
            ExtrusionBehavior::SitsOnTop { height_inches: 1.0 }
        );
    }

    #[test]
    fn builder_all_fields() {
        let id = MaterialId::new();
        let mat = Material::builder("Travertine Pavers", MaterialCategory::Hardscape)
            .id(id)
            .unit(Unit::SqFt)
            .price_per_unit(Decimal::from_str("8.50").unwrap())
            .depth_inches(1.0)
            .texture_ref("textures/travertine.pbr")
            .photo_ref("photos/travertine.jpg")
            .supplier_sku("TRAV-12x12-NAT")
            .extrusion(ExtrusionBehavior::SitsOnTop { height_inches: 1.0 })
            .build();

        assert_eq!(mat.id, id);
        assert_eq!(mat.name, "Travertine Pavers");
        assert_eq!(mat.price_per_unit, Decimal::from_str("8.50").unwrap());
        assert_eq!(mat.depth_inches, Some(1.0));
        assert_eq!(mat.supplier_sku, Some("TRAV-12x12-NAT".to_string()));
    }

    #[test]
    fn builder_default_id_is_unique() {
        let a = Material::builder("A", MaterialCategory::Fill).build();
        let b = Material::builder("B", MaterialCategory::Fill).build();
        assert_ne!(a.id, b.id);
    }
}
