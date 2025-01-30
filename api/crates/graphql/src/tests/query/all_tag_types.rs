use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
use domain::entity::tag_types::{TagType, TagTypeId};
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
    let external_services_service = MockExternalServicesServiceInterface::new();
    let media_service = MockMediaServiceInterface::new();

    let mut tags_service = MockTagsServiceInterface::new();
    tags_service
        .expect_get_tag_types()
        .times(1)
        .returning(|| {
            Box::pin(ok(vec![
                TagType {
                    id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                    slug: "character".to_string(),
                    name: "キャラクター".to_string(),
                    kana: "キャラクター".to_string(),
                },
                TagType {
                    id: TagTypeId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                    slug: "work".to_string(),
                    name: "作品".to_string(),
                    kana: "さくひん".to_string(),
                },
            ]))
        });

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, EmptyMutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .finish();

    let req = indoc! {r#"
        query {
            allTagTypes {
                id
                slug
                name
                kana
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "allTagTypes": [
            {
                "id": "44444444-4444-4444-4444-444444444444",
                "slug": "character",
                "name": "キャラクター",
                "kana": "キャラクター",
            },
            {
                "id": "66666666-6666-6666-6666-666666666666",
                "slug": "work",
                "name": "作品",
                "kana": "さくひん",
            },
        ],
    }))
}
