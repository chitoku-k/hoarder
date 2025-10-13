use async_graphql::{Schema, EmptySubscription, value};
use domain::{entity::external_services::ExternalServiceId, repository::DeleteResult};
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
    let mut external_services_service = MockExternalServicesServiceInterface::new();
    external_services_service
        .expect_delete_external_service_by_id()
        .times(1)
        .withf(|id| id == &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")))
        .returning(|_| Box::pin(ok(DeleteResult::Deleted(1))));

    let media_service = MockMediaServiceInterface::new();
    let tags_service = MockTagsServiceInterface::new();
    let normalizer = MockNormalizerInterface::new();

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface, MockQueryParserInterface>::new();
    let mutation = Mutation::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, mutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .data(normalizer)
        .finish();

    let req = indoc! {r#"
        mutation {
            deleteExternalService(id: "11111111-1111-1111-1111-111111111111") {
                deleted
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "deleteExternalService": {
            "deleted": true,
        },
    }));
}
