use axum::{
    body::{self, Body},
    http::{Method, Request},
    response::{IntoResponse, Response},
};
use futures::future::ready;
use hyper::StatusCode;
use pretty_assertions::assert_eq;
use tower::ServiceExt;

use crate::{server::Engine, service::graphql::GraphQLEndpoints};

use super::mocks::application::service::{
    graphql::MockGraphQLServiceInterface,
    objects::MockObjectsServiceInterface,
    thumbnails::MockThumbnailsServiceInterface,
};

#[tokio::test]
async fn show() {
    let expected = vec![0x01, 0x02, 0x03, 0x04];

    let mut mock_graphql_service = MockGraphQLServiceInterface::new();
    mock_graphql_service
        .expect_endpoints()
        .times(1)
        .returning(|| GraphQLEndpoints::new("/graphql", "/graphql/subscriptions"));

    let mut mock_thumbnails_service = MockThumbnailsServiceInterface::new();
    mock_thumbnails_service
        .expect_show()
        .times(1)
        .returning(move |_| {
            Box::pin(ready(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "image/webp")
                    .body(Body::from(expected.clone()))
                    .unwrap()
                    .into_response()))
        });

    let mock_objects_service = MockObjectsServiceInterface::new();

    let app = Engine::new(mock_graphql_service, mock_objects_service, mock_thumbnails_service).into_inner();
    let actual = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/thumbnails/88888888-8888-8888-8888-888888888888")
                .body(Body::empty())
                .unwrap())
        .await
        .unwrap();

    assert_eq!(actual.status(), 200);

    let expected = vec![0x01, 0x02, 0x03, 0x04];

    let actual = body::to_bytes(actual.into_body(), usize::MAX).await.unwrap();
    assert_eq!(actual.to_vec(), expected);
}
