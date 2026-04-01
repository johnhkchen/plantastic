#![cfg(feature = "integration")]
//! Integration tests for the material repository.

use pt_materials::{ExtrusionBehavior, MaterialCategory, Unit};
use pt_repo::material::{self, CreateMaterial};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;

fn sample_material(tenant_id: uuid::Uuid) -> CreateMaterial {
    CreateMaterial {
        tenant_id,
        name: "Travertine Pavers".into(),
        category: MaterialCategory::Hardscape,
        unit: Unit::SqFt,
        price_per_unit: Decimal::from_str("8.50").unwrap(),
        depth_inches: None,
        extrusion: ExtrusionBehavior::SitsOnTop { height_inches: 1.0 },
        texture_key: Some("tex/travertine.png".into()),
        photo_key: None,
        supplier_sku: Some("TRAV-12x12".into()),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_and_list_materials(pool: PgPool) {
    let tenant_id = pt_repo::tenant::create(&pool, "Material Test Co")
        .await
        .unwrap();

    let id = material::create(&pool, &sample_material(tenant_id))
        .await
        .unwrap();

    let mats = material::list_by_tenant(&pool, tenant_id).await.unwrap();
    assert_eq!(mats.len(), 1);
    assert_eq!(mats[0].id, id);
    assert_eq!(mats[0].name, "Travertine Pavers");
    assert_eq!(mats[0].category, MaterialCategory::Hardscape);
    assert_eq!(mats[0].unit, Unit::SqFt);
    assert_eq!(mats[0].price_per_unit, Decimal::from_str("8.5000").unwrap());
    assert_eq!(mats[0].supplier_sku.as_deref(), Some("TRAV-12x12"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn extrusion_jsonb_round_trip(pool: PgPool) {
    let tenant_id = pt_repo::tenant::create(&pool, "Material Test Co")
        .await
        .unwrap();

    let variants = [
        ExtrusionBehavior::SitsOnTop { height_inches: 1.5 },
        ExtrusionBehavior::Fills { flush: true },
        ExtrusionBehavior::BuildsUp {
            height_inches: 24.0,
        },
    ];

    for extrusion in &variants {
        let mut input = sample_material(tenant_id);
        input.extrusion = extrusion.clone();
        input.name = format!("Test-{extrusion:?}");

        let id = material::create(&pool, &input).await.unwrap();
        let mats = material::list_by_tenant(&pool, tenant_id).await.unwrap();
        let mat = mats.iter().find(|m| m.id == id).unwrap();
        assert_eq!(
            &mat.extrusion, extrusion,
            "extrusion round-trip failed for {extrusion:?}"
        );
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_material(pool: PgPool) {
    let tenant_id = pt_repo::tenant::create(&pool, "Material Test Co")
        .await
        .unwrap();

    let id = material::create(&pool, &sample_material(tenant_id))
        .await
        .unwrap();

    let updated = CreateMaterial {
        tenant_id,
        name: "Premium Pavers".into(),
        category: MaterialCategory::Hardscape,
        unit: Unit::SqFt,
        price_per_unit: Decimal::from_str("12.00").unwrap(),
        depth_inches: None,
        extrusion: ExtrusionBehavior::SitsOnTop { height_inches: 2.0 },
        texture_key: None,
        photo_key: None,
        supplier_sku: None,
    };

    material::update(&pool, id, &updated).await.unwrap();

    let mats = material::list_by_tenant(&pool, tenant_id).await.unwrap();
    let mat = mats.iter().find(|m| m.id == id).unwrap();
    assert_eq!(mat.name, "Premium Pavers");
    assert_eq!(mat.price_per_unit, Decimal::from_str("12.0000").unwrap());
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_nonexistent_returns_not_found(pool: PgPool) {
    let err = material::delete(&pool, uuid::Uuid::new_v4())
        .await
        .unwrap_err();
    assert!(matches!(err, pt_repo::RepoError::NotFound));
}

#[sqlx::test(migrations = "../../migrations")]
async fn depth_inches_round_trip(pool: PgPool) {
    let tenant_id = pt_repo::tenant::create(&pool, "Material Test Co")
        .await
        .unwrap();

    let mut input = sample_material(tenant_id);
    input.name = "Bark Mulch".into();
    input.category = MaterialCategory::Softscape;
    input.unit = Unit::CuYd;
    input.depth_inches = Some(4.0);
    input.extrusion = ExtrusionBehavior::Fills { flush: true };

    let id = material::create(&pool, &input).await.unwrap();
    let mats = material::list_by_tenant(&pool, tenant_id).await.unwrap();
    let mat = mats.iter().find(|m| m.id == id).unwrap();

    assert!(
        (mat.depth_inches.unwrap() - 4.0).abs() < 1e-10,
        "depth_inches should be 4.0, got {:?}",
        mat.depth_inches
    );
}
