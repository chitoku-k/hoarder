use anyhow::anyhow;
use domain::{
    entity::external_services::{ExternalService, ExternalServiceId},
    error::{Error, ErrorKind},
    repository::{external_services::MockExternalServicesRepository, DeleteResult},
    service::external_services::{ExternalServicesService, ExternalServicesServiceInterface},
};
use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};
use uuid::uuid;

#[tokio::test]
async fn create_external_service_succeeds() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_create()
        .times(1)
        .withf(|slug, name| (slug, name) == ("twitter", "Twitter"))
        .returning(|_, _| {
            Box::pin(ok(ExternalService {
                id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                slug: "twitter".to_string(),
                name: "Twitter".to_string(),
            }))
        });

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.create_external_service("twitter", "Twitter").await.unwrap();

    assert_eq!(actual, ExternalService {
        id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        slug: "twitter".to_string(),
        name: "Twitter".to_string(),
    });
}

#[tokio::test]
async fn create_external_service_fails() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_create()
        .times(1)
        .withf(|slug, name| (slug, name) == ("twitter", "Twitter"))
        .returning(|_, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.create_external_service("twitter", "Twitter").await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
async fn get_external_services_succeeds() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_fetch_all()
        .times(1)
        .returning(|| {
            Box::pin(ok(vec![
                ExternalService {
                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    slug: "pixiv".to_string(),
                    name: "pixiv".to_string(),
                },
                ExternalService {
                    id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    slug: "skeb".to_string(),
                    name: "Skeb".to_string(),
                },
                ExternalService {
                    id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    slug: "twitter".to_string(),
                    name: "Twitter".to_string(),
                },
            ]))
        });

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.get_external_services().await.unwrap();

    assert_eq!(actual, vec![
        ExternalService {
            id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            slug: "pixiv".to_string(),
            name: "pixiv".to_string(),
        },
        ExternalService {
            id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            slug: "skeb".to_string(),
            name: "Skeb".to_string(),
        },
        ExternalService {
            id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            slug: "twitter".to_string(),
            name: "Twitter".to_string(),
        },
    ]);
}

#[tokio::test]
async fn get_external_services_fails() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_fetch_all()
        .times(1)
        .returning(|| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.get_external_services().await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
async fn get_external_services_by_ids_succeeds() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids: &Vec<_>| ids == &vec![
            ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        ])
        .returning(|_| {
            Box::pin(ok(vec![
                ExternalService {
                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    slug: "pixiv".to_string(),
                    name: "pixiv".to_string(),
                },
                ExternalService {
                    id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    slug: "twitter".to_string(),
                    name: "Twitter".to_string(),
                },
            ]))
        });

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.get_external_services_by_ids(vec![
        ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
    ]).await.unwrap();

    assert_eq!(actual, vec![
        ExternalService {
            id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            slug: "pixiv".to_string(),
            name: "pixiv".to_string(),
        },
        ExternalService {
            id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            slug: "twitter".to_string(),
            name: "Twitter".to_string(),
        },
    ]);
}

#[tokio::test]
async fn get_external_services_by_ids_fails() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids: &Vec<_>| ids == &vec![
            ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        ])
        .returning(|_| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.get_external_services_by_ids(vec![
        ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
    ]).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
async fn update_external_service_by_id_succeeds() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, name| (id, name) == (
            &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            &Some("PIXIV"),
        ))
        .returning(|_, _| {
            Box::pin(ok(ExternalService {
                id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                slug: "pixiv".to_string(),
                name: "PIXIV".to_string(),
            }))
        });

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.update_external_service_by_id(
        ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        Some("PIXIV"),
    ).await.unwrap();

    assert_eq!(actual, ExternalService {
        id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        slug: "pixiv".to_string(),
        name: "PIXIV".to_string(),
    })
}

#[tokio::test]
async fn update_external_service_by_id_fails() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, name| (id, name) == (
            &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            &Some("PIXIV"),
        ))
        .returning(|_, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.update_external_service_by_id(
        ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        Some("PIXIV"),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
async fn delete_external_service_by_id_succeeds() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")))
        .returning(|_| Box::pin(ok(DeleteResult::Deleted(1))));

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.delete_external_service_by_id(
        ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
    ).await.unwrap();

    assert_eq!(actual, DeleteResult::Deleted(1));
}

#[tokio::test]
async fn delete_external_service_by_id_fails() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_delete_by_id()
        .times(1)
        .withf(|id| id == &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")))
        .returning(|_| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.delete_external_service_by_id(
        ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
