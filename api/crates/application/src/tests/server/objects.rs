use std::time::{Duration, UNIX_EPOCH};

use axum::{
    body::{self, Body},
    http::{header::LOCATION, Method, Request},
    response::{IntoResponse, Response},
};
use futures::future::ready;
use headers::{ETag, IfMatch, IfModifiedSince, IfNoneMatch};
use hyper::{StatusCode, header::{IF_MATCH, IF_MODIFIED_SINCE, IF_NONE_MATCH}};
use pretty_assertions::assert_eq;
use tower::ServiceExt;

use crate::{server::Engine, service::graphql::GraphQLEndpoints};

use super::mocks::application::service::{
    graphql::MockGraphQLServiceInterface,
    objects::MockObjectsServiceInterface,
    thumbnails::MockThumbnailsServiceInterface,
};

#[tokio::test]
async fn serve_succeeds() {
    let mut mock_graphql_service = MockGraphQLServiceInterface::new();
    mock_graphql_service
        .expect_endpoints()
        .times(1)
        .returning(|| GraphQLEndpoints::new("/graphql", "/graphql/subscriptions"));

    let mut mock_objects_service = MockObjectsServiceInterface::new();
    mock_objects_service
        .expect_serve()
        .times(1)
        .withf(|url, precondition| url == "file:///77777777-7777-7777-7777-777777777777.png" && precondition.is_none())
        .returning(|_, _| {
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

#[tokio::test]
async fn serve_succeeds_with_if_none_match() {
    let mut mock_graphql_service = MockGraphQLServiceInterface::new();
    mock_graphql_service
        .expect_endpoints()
        .times(1)
        .returning(|| GraphQLEndpoints::new("/graphql", "/graphql/subscriptions"));

    let mut mock_objects_service = MockObjectsServiceInterface::new();
    mock_objects_service
        .expect_serve()
        .times(1)
        .withf(|url, precondition| {
            let expected_url = "file:///77777777-7777-7777-7777-777777777777.png" ;
            let expected_precondition = IfNoneMatch::from(r#""2710-5e06bafe9a240""#.parse::<ETag>().unwrap()).into();
            url == expected_url && precondition.as_ref().is_some_and(|precondition| precondition == &expected_precondition)
        })
        .returning(|_, _| {
            Box::pin(ready(
                Response::builder()
                    .status(StatusCode::NOT_MODIFIED)
                    .body(Body::from(()))
                    .unwrap()
                    .into_response()))
        });

    let mock_thumbnails_service = MockThumbnailsServiceInterface::new();

    let app = Engine::new(mock_graphql_service, mock_objects_service, mock_thumbnails_service).into_inner();
    let actual = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .header(IF_NONE_MATCH, r#""2710-5e06bafe9a240""#)
                .uri("/objects?url=file%3A%2F%2F%2F77777777-7777-7777-7777-777777777777.png")
                .body(Body::empty())
                .unwrap())
        .await
        .unwrap();

    assert_eq!(actual.status(), 304);
}

#[tokio::test]
async fn serve_succeeds_with_if_match() {
    let mut mock_graphql_service = MockGraphQLServiceInterface::new();
    mock_graphql_service
        .expect_endpoints()
        .times(1)
        .returning(|| GraphQLEndpoints::new("/graphql", "/graphql/subscriptions"));

    let mut mock_objects_service = MockObjectsServiceInterface::new();
    mock_objects_service
        .expect_serve()
        .times(1)
        .withf(|url, precondition| {
            let expected_url = "file:///77777777-7777-7777-7777-777777777777.png" ;
            let expected_precondition = IfMatch::from(r#""2710-5e06bafe9a240""#.parse::<ETag>().unwrap()).into();
            url == expected_url && precondition.as_ref().is_some_and(|precondition| precondition == &expected_precondition)
        })
        .returning(|_, _| {
            Box::pin(ready(
                Response::builder()
                    .status(StatusCode::NOT_MODIFIED)
                    .body(Body::from(()))
                    .unwrap()
                    .into_response()))
        });

    let mock_thumbnails_service = MockThumbnailsServiceInterface::new();

    let app = Engine::new(mock_graphql_service, mock_objects_service, mock_thumbnails_service).into_inner();
    let actual = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .header(IF_MATCH, r#""2710-5e06bafe9a240""#)
                .uri("/objects?url=file%3A%2F%2F%2F77777777-7777-7777-7777-777777777777.png")
                .body(Body::empty())
                .unwrap())
        .await
        .unwrap();

    assert_eq!(actual.status(), 304);
}

#[tokio::test]
async fn serve_succeeds_with_if_modified_since() {
    let mut mock_graphql_service = MockGraphQLServiceInterface::new();
    mock_graphql_service
        .expect_endpoints()
        .times(1)
        .returning(|| GraphQLEndpoints::new("/graphql", "/graphql/subscriptions"));

    let mut mock_objects_service = MockObjectsServiceInterface::new();
    mock_objects_service
        .expect_serve()
        .times(1)
        .withf(|url, precondition| {
            let expected_url = "file:///77777777-7777-7777-7777-777777777777.png" ;
            let expected_precondition = IfModifiedSince::from(UNIX_EPOCH + Duration::from_secs(1654128001)).into();
            url == expected_url && precondition.as_ref().is_some_and(|precondition| precondition == &expected_precondition)
        })
        .returning(|_, _| {
            Box::pin(ready(
                Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(&[0x01, 0x02, 0x03, 0x04][..]))
                    .unwrap()
                    .into_response()))
        });

    let mock_thumbnails_service = MockThumbnailsServiceInterface::new();

    let app = Engine::new(mock_graphql_service, mock_objects_service, mock_thumbnails_service).into_inner();
    let actual = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .header(IF_MODIFIED_SINCE, "Thu, 02 Jun 2022 00:00:01 GMT")
                .uri("/objects?url=file%3A%2F%2F%2F77777777-7777-7777-7777-777777777777.png")
                .body(Body::empty())
                .unwrap())
        .await
        .unwrap();

    assert_eq!(actual.status(), 200);

    let actual = body::to_bytes(actual.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&actual, &[0x01, 0x02, 0x03, 0x04][..]);
}
