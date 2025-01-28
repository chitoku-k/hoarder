use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
use domain::entity::objects::{Entry, EntryKind, EntryUrl, EntryUrlPath};
use futures::future::ok;
use graphql::query::Query;
use indoc::indoc;
use pretty_assertions::assert_eq;

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
    let external_services_service = MockExternalServicesServiceInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_get_objects()
        .times(1)
        .withf(|prefix, kind| (prefix, kind) == (
            &EntryUrlPath::from("/path/to/dest".to_string()),
            &Some(EntryKind::Container),
        ))
        .returning(|_, _| {
            Box::pin(ok(vec![
                Entry::new(
                    "container01".to_string(),
                    Some(EntryUrl::from("file:///path/to/dest/container01".to_string())),
                    EntryKind::Container,
                    None,
                ),
                Entry::new(
                    "container02".to_string(),
                    Some(EntryUrl::from("file:///path/to/dest/container02".to_string())),
                    EntryKind::Container,
                    None,
                ),
            ]))
        });

    let tags_service = MockTagsServiceInterface::new();

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, EmptyMutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .finish();

    let req = indoc! {r#"
        query {
            objects(prefix: "/path/to/dest", kind: CONTAINER) {
                name
                url
                kind
                metadata {
                    size
                    createdAt
                    updatedAt
                    accessedAt
                }
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "objects": [
            {
                "name": "container01",
                "url": "file:///path/to/dest/container01",
                "kind": "CONTAINER",
                "metadata": null,
            },
            {
                "name": "container02",
                "url": "file:///path/to/dest/container02",
                "kind": "CONTAINER",
                "metadata": null,
            },
        ],
    }));
}
