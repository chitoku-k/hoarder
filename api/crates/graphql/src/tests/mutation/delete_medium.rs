use async_graphql::{Schema, EmptySubscription, value};
use domain::{entity::media::MediumId, repository::DeleteResult};
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
        .expect_delete_medium_by_id()
        .times(1)
        .withf(|id, delete_objects| (id, delete_objects) == (
            &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
            &true,
        ))
        .returning(|_, _| Box::pin(ok(DeleteResult::Deleted(1))));

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
            deleteMedium(
                id: "77777777-7777-7777-7777-777777777777",
                deleteObjects: true,
            ) {
                deleted
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "deleteMedium": {
            "deleted": true,
        },
    }));
}
