use async_graphql::{Schema, EmptyMutation, EmptySubscription, value};
use domain::{
    entity::external_services::{self, ExternalServiceId},
    service::{
        external_services::MockExternalServicesServiceInterface,
        media::MockMediaServiceInterface,
        tags::MockTagsServiceInterface,
    },
};
use graphql::query::Query;
use indoc::indoc;
use pretty_assertions::assert_eq;
use uuid::{uuid, Uuid};

// Concrete type is required both in implementation and expectation.
type IntoIterMap<T, U> = std::iter::Map<std::vec::IntoIter<T>, fn(T) -> U>;

#[tokio::test]
async fn all_external_services_succeeds() {
    let mut external_services_service = MockExternalServicesServiceInterface::new();
    external_services_service
        .expect_get_external_services()
        .times(1)
        .returning(|| {
            Ok(vec![
                external_services::ExternalService {
                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    slug: "pixiv".to_string(),
                    name: "pixiv".to_string(),
                },
                external_services::ExternalService {
                    id: ExternalServiceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    slug: "skeb".to_string(),
                    name: "Skeb".to_string(),
                },
                external_services::ExternalService {
                    id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    slug: "twitter".to_string(),
                    name: "Twitter".to_string(),
                },
            ])
        });

    let media_service = MockMediaServiceInterface::new();
    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            allExternalServices {
                id
                slug
                name
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "allExternalServices": [
            {
                "id": "11111111-1111-1111-1111-111111111111",
                "slug": "pixiv",
                "name": "pixiv",
            },
            {
                "id": "22222222-2222-2222-2222-222222222222",
                "slug": "skeb",
                "name": "Skeb",
            },
            {
                "id": "33333333-3333-3333-3333-333333333333",
                "slug": "twitter",
                "name": "Twitter",
            },
        ],
    }));
}

#[tokio::test]
async fn external_services_succeeds() {
    let mut external_services_service = MockExternalServicesServiceInterface::new();
    external_services_service
        .expect_get_external_services_by_ids::<IntoIterMap<Uuid, ExternalServiceId>>()
        .times(1)
        .withf(|ids| ids.clone().eq([
            ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        ]))
        .returning(|_| {
            Ok(vec![
                external_services::ExternalService {
                    id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    slug: "pixiv".to_string(),
                    name: "pixiv".to_string(),
                },
                external_services::ExternalService {
                    id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    slug: "twitter".to_string(),
                    name: "Twitter".to_string(),
                },
            ])
        });

    let media_service = MockMediaServiceInterface::new();
    let tags_service = MockTagsServiceInterface::new();

    let query = Query::new(external_services_service, media_service, tags_service);
    let schema = Schema::build(query, EmptyMutation, EmptySubscription).finish();
    let req = indoc! {r#"
        query {
            externalServices(ids: ["11111111-1111-1111-1111-111111111111", "33333333-3333-3333-3333-333333333333"]) {
                id
                slug
                name
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
            },
            {
                "id": "33333333-3333-3333-3333-333333333333",
                "slug": "twitter",
                "name": "Twitter",
            },
        ],
    }));
}
