use anyhow::anyhow;
use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};
use uuid::uuid;

use crate::{
    entity::external_services::{ExternalService, ExternalServiceId},
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
            ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        ]))
        .returning(|_| {
            Box::pin(ok(vec![
                ExternalService {
                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    slug: "pixiv".to_string(),
                    kind: "pixiv".to_string(),
                    name: "pixiv".to_string(),
                    base_url: Some("https://www.pixiv.net".to_string()),
                    url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
                },
                ExternalService {
                    id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    slug: "x".to_string(),
                    kind: "x".to_string(),
                    name: "X".to_string(),
                    base_url: Some("https://x.com".to_string()),
                    url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
                },
            ]))
        });

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.get_external_services_by_ids([
        ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
    ].into_iter()).await.unwrap();

    assert_eq!(actual, vec![
        ExternalService {
            id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            slug: "pixiv".to_string(),
            kind: "pixiv".to_string(),
            name: "pixiv".to_string(),
            base_url: Some("https://www.pixiv.net".to_string()),
            url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
        },
        ExternalService {
            id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            slug: "x".to_string(),
            kind: "x".to_string(),
            name: "X".to_string(),
            base_url: Some("https://x.com".to_string()),
            url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
        },
    ]);
}

#[tokio::test]
async fn fails() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_fetch_by_ids()
        .times(1)
        .withf(|ids| ids.clone_box().eq([
            ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        ]))
        .returning(|_| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.get_external_services_by_ids([
        ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
    ].into_iter()).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
