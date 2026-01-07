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
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids| ids.clone_box().eq([
            ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        ]))
        .returning(|_| {
            Box::pin(ok(vec![
                ExternalService {
                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    slug: "pixiv".to_string(),
                    kind: ExternalServiceKind::Pixiv,
                    name: "pixiv".to_string(),
                    base_url: Some("https://www.pixiv.net".to_string()),
                    url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
                },
            ]))
        });

    mock_external_services_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, slug, name, base_url, url_pattern| (id, slug, name, base_url, url_pattern) == (
            &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            &None,
            &Some("PIXIV"),
            &None,
            &None,
        ))
        .returning(|_, _, _, _, _| {
            Box::pin(ok(ExternalService {
                id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                slug: "pixiv".to_string(),
                kind: ExternalServiceKind::Pixiv,
                name: "PIXIV".to_string(),
                base_url: Some("https://www.pixiv.net".to_string()),
                url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
            }))
        });

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.update_external_service_by_id(
        ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        None,
        Some("PIXIV"),
        None,
        None,
    ).await.unwrap();

    assert_eq!(actual, ExternalService {
        id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        slug: "pixiv".to_string(),
        kind: ExternalServiceKind::Pixiv,
        name: "PIXIV".to_string(),
        base_url: Some("https://www.pixiv.net".to_string()),
        url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
    })
}

#[tokio::test]
async fn succeeds_with_reset() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids| ids.clone_box().eq([
            ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        ]))
        .returning(|_| {
            Box::pin(ok(vec![
                ExternalService {
                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    slug: "pixiv".to_string(),
                    kind: ExternalServiceKind::Pixiv,
                    name: "pixiv".to_string(),
                    base_url: Some("https://www.pixiv.net".to_string()),
                    url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
                },
            ]))
        });

    mock_external_services_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, slug, name, base_url, url_pattern| (id, slug, name, base_url, url_pattern) == (
            &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            &None,
            &Some("PIXIV"),
            &Some(Some("https://www.pixiv.net")),
            &Some(Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$")),
        ))
        .returning(|_, _, _, _, _| {
            Box::pin(ok(ExternalService {
                id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                slug: "pixiv".to_string(),
                kind: ExternalServiceKind::Pixiv,
                name: "PIXIV".to_string(),
                base_url: Some("https://www.pixiv.net".to_string()),
                url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
            }))
        });

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.update_external_service_by_id(
        ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        None,
        Some("PIXIV"),
        Some(None),
        Some(None),
    ).await.unwrap();

    assert_eq!(actual, ExternalService {
        id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        slug: "pixiv".to_string(),
        kind: ExternalServiceKind::Pixiv,
        name: "PIXIV".to_string(),
        base_url: Some("https://www.pixiv.net".to_string()),
        url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
    })
}

#[tokio::test]
async fn fails() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids| ids.clone_box().eq([
            ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        ]))
        .returning(|_| {
            Box::pin(ok(vec![
                ExternalService {
                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    slug: "pixiv".to_string(),
                    kind: ExternalServiceKind::Pixiv,
                    name: "pixiv".to_string(),
                    base_url: Some("https://www.pixiv.net".to_string()),
                    url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
                },
            ]))
        });

    mock_external_services_repository
        .expect_update_by_id()
        .times(1)
        .withf(|id, slug, name, base_url, url_pattern| (id, slug, name, base_url, url_pattern) == (
            &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            &None,
            &Some("PIXIV"),
            &None,
            &None,
        ))
        .returning(|_, _, _, _, _| Box::pin(err(Error::other("error communicating with database"))));

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.update_external_service_by_id(
        ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        None,
        Some("PIXIV"),
        None,
        None,
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}

#[tokio::test]
async fn fails_with_not_found() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids| ids.clone_box().eq([
            ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        ]))
        .returning(|_| Box::pin(ok(Vec::new())));

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.update_external_service_by_id(
        ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        None,
        Some("PIXIV"),
        None,
        None,
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::ExternalServiceNotFound { id } if id == &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")));
}

#[tokio::test]
async fn fails_with_url_pattern_invalid() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids| ids.clone_box().eq([
            ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        ]))
        .returning(|_| {
            Box::pin(ok(vec![
                ExternalService {
                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    slug: "pixiv".to_string(),
                    kind: ExternalServiceKind::Pixiv,
                    name: "pixiv".to_string(),
                    base_url: Some("https://www.pixiv.net".to_string()),
                    url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
                },
            ]))
        });

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.update_external_service_by_id(
        ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        None,
        Some("PIXIV"),
        None,
        Some(Some("(")),
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::ExternalServiceUrlPatternInvalid { url_pattern, description } if url_pattern == "(" && !description.as_ref().unwrap().is_empty());
}
