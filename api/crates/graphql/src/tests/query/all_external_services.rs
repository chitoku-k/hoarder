use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
use domain::entity::external_services::{ExternalService, ExternalServiceId, ExternalServiceKind};
use futures::future::ok;
use indoc::indoc;
use pretty_assertions::assert_eq;
use uuid::uuid;

use crate::query::Query;

use super::mocks::{
    domain::service::{
        external_services::MockExternalServicesServiceInterface,
        media::MockMediaServiceInterface,
        tags::MockTagsServiceInterface,
    },
    normalizer::MockNormalizerInterface,
};

#[tokio::test]
async fn succeeds() {
    let mut external_services_service = MockExternalServicesServiceInterface::new();
    external_services_service
        .expect_get_external_services()
        .times(1)
        .returning(|| {
            Box::pin(ok(vec![
                ExternalService {
                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    slug: "pixiv".to_string(),
                    kind: ExternalServiceKind::Pixiv,
                    name: "pixiv".to_string(),
                    base_url: Some("https://www.pixiv.net".to_string()),
                    url_pattern: Some(r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$".to_string()),
                },
                ExternalService {
                    id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    slug: "skeb".to_string(),
                    kind: ExternalServiceKind::Skeb,
                    name: "Skeb".to_string(),
                    base_url: Some("https://skeb.jp".to_string()),
                    url_pattern: Some(r"^https?://skeb\.jp/@(?<creatorId>[^/]+)/works/(?<id>\d+)(?:[?#].*)?$".to_string()),
                },
                ExternalService {
                    id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    slug: "x".to_string(),
                    kind: ExternalServiceKind::X,
                    name: "X".to_string(),
                    base_url: Some("https://x.com".to_string()),
                    url_pattern: Some(r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$".to_string()),
                },
            ]))
        });

    let media_service = MockMediaServiceInterface::new();
    let tags_service = MockTagsServiceInterface::new();

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, EmptyMutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .finish();

    let req = indoc! {r#"
        query {
            allExternalServices {
                id
                slug
                kind
                name
                baseUrl
                urlPattern
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "allExternalServices": [
            {
                "id": "11111111-1111-1111-1111-111111111111",
                "slug": "pixiv",
                "kind": "pixiv",
                "name": "pixiv",
                "baseUrl": "https://www.pixiv.net",
                "urlPattern": r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$",
            },
            {
                "id": "22222222-2222-2222-2222-222222222222",
                "slug": "skeb",
                "kind": "skeb",
                "name": "Skeb",
                "baseUrl": "https://skeb.jp",
                "urlPattern": r"^https?://skeb\.jp/@(?<creatorId>[^/]+)/works/(?<id>\d+)(?:[?#].*)?$",
            },
            {
                "id": "33333333-3333-3333-3333-333333333333",
                "slug": "x",
                "kind": "x",
                "name": "X",
                "baseUrl": "https://x.com",
                "urlPattern": r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$",
            },
        ],
    }));
}
