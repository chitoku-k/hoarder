use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};
use uuid::uuid;

use crate::{
    entity::external_services::{ExternalService, ExternalServiceId, ExternalServiceKind},
    error::{Error, ErrorKind},
    service::external_services::{ExternalServicesService, ExternalServicesServiceInterface},
};

use super::mocks::domain::repository::external_services::MockExternalServicesRepository;

#[tokio::test]
async fn succeeds() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_create()
        .times(1)
        .withf(|slug, kind, name, base_url, url_pattern| (slug, kind, name, base_url, url_pattern) == (
            "x",
            &ExternalServiceKind::X,
            "X",
            &Some("https://x.com"),
            &Some(r"^https?://(?:x\.com|twitter\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$"),
        ))
        .returning(|_, _, _, _, _| {
            Box::pin(ok(ExternalService {
                id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                slug: "x".to_string(),
                kind: ExternalServiceKind::X,
                name: "X".to_string(),
                base_url: Some("https://x.com".to_string()),
                url_pattern: Some(r"^https?://(?:x\.com|twitter\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
            }))
        });

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.create_external_service(
        "x",
        ExternalServiceKind::X,
        "X",
        None,
        None,
    ).await.unwrap();

    assert_eq!(actual, ExternalService {
        id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        slug: "x".to_string(),
        kind: ExternalServiceKind::X,
        name: "X".to_string(),
        base_url: Some("https://x.com".to_string()),
        url_pattern: Some(r"^https?://(?:x\.com|twitter\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
    });
}

#[tokio::test]
async fn succeeds_with_url_pattern() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_create()
        .times(1)
        .withf(|slug, kind, name, base_url, url_pattern| (slug, kind, name, base_url, url_pattern) == (
            "x",
            &ExternalServiceKind::X,
            "X",
            &Some("https://x.com"),
            &Some(r"^https?://x\.com/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$"),
        ))
        .returning(|_, _, _, _, _| {
            Box::pin(ok(ExternalService {
                id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                slug: "x".to_string(),
                kind: ExternalServiceKind::X,
                name: "X".to_string(),
                base_url: Some("https://x.com".to_string()),
                url_pattern: Some(r"^https?://x\.com/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
            }))
        });

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.create_external_service(
        "x",
        ExternalServiceKind::X,
        "X",
        Some("https://x.com"),
        Some(r"^https?://x\.com/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$"),
    ).await.unwrap();

    assert_eq!(actual, ExternalService {
        id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        slug: "x".to_string(),
        kind: ExternalServiceKind::X,
        name: "X".to_string(),
        base_url: Some("https://x.com".to_string()),
        url_pattern: Some(r"^https?://x\.com/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
    });
}

#[tokio::test]
async fn fails() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_create()
        .times(1)
        .withf(|slug, kind, name, base_url, url_pattern| (slug, kind, name, base_url, url_pattern) == (
            "x",
            &ExternalServiceKind::X,
            "X",
            &Some("https://x.com"),
            &Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$"),
        ))
        .returning(|_, _, _, _, _| Box::pin(err(Error::other("error communicating with database"))));

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.create_external_service(
        "x",
        ExternalServiceKind::X,
        "X",
        Some("https://x.com"),
        Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$"),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
async fn fails_with_url_pattern_invalid() {
    let mock_external_services_repository = MockExternalServicesRepository::new();

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.create_external_service(
        "x",
        ExternalServiceKind::X,
        "X",
        Some("https://x.com"),
        Some("("),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::ExternalServiceUrlPatternInvalid { url_pattern, description } if url_pattern == "(" && !description.as_ref().unwrap().is_empty());
}
