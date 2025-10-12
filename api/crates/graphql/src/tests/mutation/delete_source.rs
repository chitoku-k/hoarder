use std::sync::Arc;

use async_graphql::{Schema, EmptySubscription, value};
use domain::{entity::sources::SourceId, repository::DeleteResult};
use futures::future::ok;
use indoc::indoc;
use pretty_assertions::assert_eq;
use uuid::uuid;

use crate::{mutation::Mutation, query::Query};

use super::mocks::{
    domain::service::{
        external_services::MockExternalServicesServiceInterface,
        media::MockMediaServiceInterface,
        tags::MockTagsServiceInterface,
    },
    normalizer::MockNormalizerInterface,
    query::MockQueryParserInterface,
};

#[tokio::test]
async fn succeeds() {
    let external_services_service = MockExternalServicesServiceInterface::new();
    let tags_service = MockTagsServiceInterface::new();
    let normalizer = MockNormalizerInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_delete_source_by_id()
        .times(1)
        .withf(|id| id == &SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")))
        .returning(|_| Box::pin(ok(DeleteResult::Deleted(1))));

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface, MockQueryParserInterface>::new();
    let mutation = Mutation::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, mutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .data(Arc::new(normalizer))
        .finish();

    let req = indoc! {r#"
        mutation {
            deleteSource(id: "11111111-1111-1111-1111-111111111111") {
                deleted
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "deleteSource": {
            "deleted": true,
        },
    }));
}
