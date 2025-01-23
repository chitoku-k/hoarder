use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
use domain::entity::external_services::{ExternalService, ExternalServiceId};
use futures::future::ok;
use graphql::query::Query;
use indoc::indoc;
use pretty_assertions::assert_eq;
use uuid::uuid;

mod mocks;
use mocks::{
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
        .expect_get_external_services_by_ids()
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
            externalServices(ids: ["11111111-1111-1111-1111-111111111111", "33333333-3333-3333-3333-333333333333"]) {
                id
                slug
                name
                baseUrl
                urlPattern
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "externalServices": [
            {
                "id": "11111111-1111-1111-1111-111111111111",
                "slug": "pixiv",
                "name": "pixiv",
                "baseUrl": "https://www.pixiv.net",
                "urlPattern": r"^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$",
            },
            {
                "id": "33333333-3333-3333-3333-333333333333",
                "slug": "x",
                "name": "X",
                "baseUrl": "https://x.com",
                "urlPattern": r"^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$",
            },
        ],
    }));
}
