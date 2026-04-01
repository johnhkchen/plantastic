#![cfg(feature = "integration")]
//! Integration tests for the project repository.

use pt_project::ProjectStatus;
use pt_repo::project::{self, CreateProject};
use sqlx::PgPool;

#[sqlx::test(migrations = "../../migrations")]
async fn create_and_get_project(pool: PgPool) {
    let tenant_id = pt_repo::tenant::create(&pool, "Test Co").await.unwrap();

    let id = project::create(
        &pool,
        &CreateProject {
            tenant_id,
            client_name: Some("Alice".into()),
            client_email: Some("alice@example.com".into()),
            address: Some("123 Main St".into()),
        },
    )
    .await
    .unwrap();

    let row = project::get_by_id(&pool, id).await.unwrap();
    assert_eq!(row.id, id);
    assert_eq!(row.tenant_id, tenant_id);
    assert_eq!(row.client_name.as_deref(), Some("Alice"));
    assert_eq!(row.status, ProjectStatus::Draft);

    // Verify tenant still exists after project operations
    pt_repo::tenant::get_by_id(&pool, tenant_id).await.unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_by_tenant(pool: PgPool) {
    let tenant_id = pt_repo::tenant::create(&pool, "Test Co").await.unwrap();

    let id1 = project::create(
        &pool,
        &CreateProject {
            tenant_id,
            client_name: Some("Bob".into()),
            client_email: None,
            address: None,
        },
    )
    .await
    .unwrap();

    let id2 = project::create(
        &pool,
        &CreateProject {
            tenant_id,
            client_name: Some("Carol".into()),
            client_email: None,
            address: None,
        },
    )
    .await
    .unwrap();

    let projects = project::list_by_tenant(&pool, tenant_id).await.unwrap();
    let ids: Vec<_> = projects.iter().map(|p| p.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_status_valid_transition(pool: PgPool) {
    let tenant_id = pt_repo::tenant::create(&pool, "Test Co").await.unwrap();

    let id = project::create(
        &pool,
        &CreateProject {
            tenant_id,
            client_name: None,
            client_email: None,
            address: None,
        },
    )
    .await
    .unwrap();

    project::update_status(&pool, id, ProjectStatus::Quoted)
        .await
        .unwrap();
    let row = project::get_by_id(&pool, id).await.unwrap();
    assert_eq!(row.status, ProjectStatus::Quoted);
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_status_invalid_transition(pool: PgPool) {
    let tenant_id = pt_repo::tenant::create(&pool, "Test Co").await.unwrap();

    let id = project::create(
        &pool,
        &CreateProject {
            tenant_id,
            client_name: None,
            client_email: None,
            address: None,
        },
    )
    .await
    .unwrap();

    // Draft -> Complete is invalid
    let err = project::update_status(&pool, id, ProjectStatus::Complete)
        .await
        .unwrap_err();
    assert!(matches!(err, pt_repo::RepoError::Conflict(_)));
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_and_get_returns_not_found(pool: PgPool) {
    let tenant_id = pt_repo::tenant::create(&pool, "Test Co").await.unwrap();

    let id = project::create(
        &pool,
        &CreateProject {
            tenant_id,
            client_name: None,
            client_email: None,
            address: None,
        },
    )
    .await
    .unwrap();

    project::delete(&pool, id).await.unwrap();

    let err = project::get_by_id(&pool, id).await.unwrap_err();
    assert!(matches!(err, pt_repo::RepoError::NotFound));
}
