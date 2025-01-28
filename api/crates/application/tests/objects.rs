use application::{server::Engine, service::graphql::GraphQLEndpoints};
use axum::{
    body::Body,
    http::{header::LOCATION, Method, Request},
    response::{IntoResponse, Response},
};
use futures::future::ready;
use hyper::StatusCode;
use pretty_assertions::assert_eq;
use tower::ServiceExt;

mod mocks;
use mocks::application::service::{
    graphql::MockGraphQLServiceInterface,
    objects::MockObjectsServiceInterface,
    thumbnails::MockThumbnailsServiceInterface,
};

#[tokio::test]
async fn redirect() {
    let mut mock_graphql_service = MockGraphQLServiceInterface::new();
    mock_graphql_service
        .expect_endpoints()
        .times(1)
        .returning(|| GraphQLEndpoints::new("/graphql", "/graphql/subscriptions"));

    let mut mock_objects_service = MockObjectsServiceInterface::new();
    mock_objects_service
        .expect_redirect()
        .times(1)
        .withf(|url| url == "file:///77777777-7777-7777-7777-777777777777.png")
        .returning(|_| {
            Box::pin(ready(
                Response::builder()
                    .status(StatusCode::FOUND)
                    .header(LOCATION, "https://original.example.com/77777777-7777-7777-7777-777777777777.png")
                    .body(Body::empty())
                    .unwrap()
                    .into_response()))
        });

    let mock_thumbnails_service = MockThumbnailsServiceInterface::new();

    let app = Engine::new(mock_graphql_service, mock_objects_service, mock_thumbnails_service).into_inner();
    let actual = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/objects?url=file%3A%2F%2F%2F77777777-7777-7777-7777-777777777777.png")
                .body(Body::empty())
                .unwrap())
        .await
        .unwrap();

    assert_eq!(actual.status(), 302);
    assert_eq!(actual.headers()[LOCATION], "https://original.example.com/77777777-7777-7777-7777-777777777777.png");
}
