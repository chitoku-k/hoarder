use anyhow::anyhow;
use futures::future::{err, ok};
use pretty_assertions::{assert_eq, assert_matches};
use uuid::uuid;

use crate::{
    entity::external_services::{ExternalMetadata, ExternalService, ExternalServiceId},
    error::{Error, ErrorKind},
    service::external_services::{ExternalServicesService, ExternalServicesServiceInterface},
};

use super::mocks::domain::repository::external_services::MockExternalServicesRepository;

#[tokio::test]
async fn succeeds() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_fetch_all()
        .times(1)
        .returning(|| {
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
                    id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    slug: "skeb".to_string(),
                    kind: "skeb".to_string(),
                    name: "Skeb".to_string(),
                    base_url: Some("https://skeb.jp".to_string()),
                    url_pattern: Some(r"^https?://skeb\.jp/@(?<creatorId>[^/]+)/works/(?<id>\d+)(?:[?#].*)?$".to_string()),
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
    let actual = service.get_external_services_by_url("https://x.com/_namori_/status/727620202049900544").await.unwrap();

    assert_eq!(actual, vec![
        (
            ExternalService {
                id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                slug: "x".to_string(),
                kind: "x".to_string(),
                name: "X".to_string(),
                base_url: Some("https://x.com".to_string()),
                url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
            },
            ExternalMetadata::X {
                id: 727620202049900544,
                creator_id: Some("_namori_".to_string()),
            },
        ),
    ]);
}

#[tokio::test]
async fn fails() {
    let mut mock_external_services_repository = MockExternalServicesRepository::new();
    mock_external_services_repository
        .expect_fetch_all()
        .times(1)
        .returning(|| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let service = ExternalServicesService::new(mock_external_services_repository);
    let actual = service.get_external_services_by_url("https://x.com/_namori_/status/727620202049900544").await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
