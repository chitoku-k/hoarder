use std::sync::Arc;

use async_graphql::{Schema, EmptySubscription, value};
use domain::{entity::tags::TagId, repository::DeleteResult};
use futures::future::ok;
use graphql::{mutation::Mutation, query::Query};
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
    let external_services_service = MockExternalServicesServiceInterface::new();
    let media_service = MockMediaServiceInterface::new();
    let normalizer = MockNormalizerInterface::new();

    let mut tags_service = MockTagsServiceInterface::new();
    tags_service
        .expect_delete_tag_by_id()
        .times(1)
        .withf(|id, recursive| id == &TagId::from(uuid!("33333333-3333-3333-3333-333333333333")) && *recursive)
        .returning(|_, _| Box::pin(ok(DeleteResult::Deleted(1))));

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let mutation = Mutation::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, mutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .data(Arc::new(normalizer))
        .finish();

    let req = indoc! {r#"
        mutation {
            deleteTag(
                id: "33333333-3333-3333-3333-333333333333",
                recursive: true,
            ) {
                deleted
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "deleteTag": {
            "deleted": true,
        },
    }));
}
