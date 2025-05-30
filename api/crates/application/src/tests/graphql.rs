use axum::{
    body::{self, Body},
    http::{Method, Request},
    response::{IntoResponse, Response},
};
use futures::future::ready;
use hyper::StatusCode;
use indoc::indoc;
use pretty_assertions::assert_eq;
use serde_json::json;
use tower::ServiceExt;
use tungstenite::handshake::{client::generate_key, server::create_response_with_body};

use crate::{server::Engine, service::graphql::GraphQLEndpoints};

use super::mocks::application::service::{
    graphql::MockGraphQLServiceInterface,
    objects::MockObjectsServiceInterface,
    thumbnails::MockThumbnailsServiceInterface,
};

#[tokio::test]
async fn graphql() {
    let expected = json!({
        "data": {
            "externalServices": [
                {
                    "id": "11111111-1111-1111-1111-111111111111",
                    "slug": "pixiv",
                    "name": "pixiv",
                },
                {
                    "id": "33333333-3333-3333-3333-333333333333",
                    "slug": "x",
                    "name": "X",
                },
            ],
        },
    });

    let query = indoc! {r#"
        query {
            externalServices(ids: ["11111111-1111-1111-1111-111111111111", "33333333-3333-3333-3333-333333333333"]) {
                id
                slug
                name
            }
        }
    "#};
    let req = json!({
        "query": query,
    });

    let mut mock_graphql_service = MockGraphQLServiceInterface::new();
    mock_graphql_service
        .expect_endpoints()
        .times(1)
        .returning(|| GraphQLEndpoints::new("/graphql", "/graphql/subscriptions"));

    mock_graphql_service
        .expect_execute()
        .times(1)
        .returning(move |_| {
            Box::pin(ready(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(Body::new(expected.to_string()))
                    .unwrap()
                    .into_response()))
        });

    let mock_objects_service = MockObjectsServiceInterface::new();
    let mock_thumbnails_service = MockThumbnailsServiceInterface::new();

    let app = Engine::new(mock_graphql_service, mock_objects_service, mock_thumbnails_service).into_inner();
    let actual = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/graphql")
                .header("Content-Type", "application/json")
                .body(Body::new(req.to_string()))
                .unwrap())
        .await
        .unwrap();

    assert_eq!(actual.status(), 200);

    let expected = json!({
        "data": {
            "externalServices": [
                {
                    "id": "11111111-1111-1111-1111-111111111111",
                    "slug": "pixiv",
                    "name": "pixiv",
                },
                {
                    "id": "33333333-3333-3333-3333-333333333333",
                    "slug": "x",
                    "name": "X",
                },
            ],
        },
    });

    let actual = body::to_bytes(actual.into_body(), usize::MAX).await.unwrap();
    let actual = String::from_utf8(actual.to_vec()).unwrap();
    assert_eq!(actual, expected.to_string());
}

#[tokio::test]
async fn graphql_subscriptions() {
    let mut mock_graphql_service = MockGraphQLServiceInterface::new();
    mock_graphql_service
        .expect_endpoints()
        .times(1)
        .returning(|| GraphQLEndpoints::new("/graphql", "/graphql/subscriptions"));

    mock_graphql_service
        .expect_subscriptions()
        .times(1)
        .returning(|req| Box::pin(ready(create_response_with_body(&req, Body::empty).unwrap())));

    let mock_objects_service = MockObjectsServiceInterface::new();
    let mock_thumbnails_service = MockThumbnailsServiceInterface::new();

    let app = Engine::new(mock_graphql_service, mock_objects_service, mock_thumbnails_service).into_inner();
    let actual = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/graphql/subscriptions")
                .header("Connection", "Upgrade")
                .header("Upgrade", "websocket")
                .header("Sec-WebSocket-Version", "13")
                .header("Sec-WebSocket-Key", generate_key())
                .header("Sec-WebSocket-Protocol", "graphql-transport-ws")
                .body(Body::empty())
                .unwrap())
        .await
        .unwrap();

    assert_eq!(actual.status(), 101);
}

#[tokio::test]
async fn graphiql() {
    let expected = indoc! {r#"
        <!DOCTYPE html>
        <html lang="en">
            <meta charset="utf-8">
            <title>GraphiQL IDE</title>
        </html>
    "#};

    let mut mock_graphql_service = MockGraphQLServiceInterface::new();
    mock_graphql_service
        .expect_endpoints()
        .times(1)
        .returning(|| GraphQLEndpoints::new("/graphql", "/graphql/subscriptions"));

    mock_graphql_service
        .expect_graphiql()
        .times(1)
        .returning(move || {
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html; charset=utf-8")
                .body(Body::from(expected.to_string()))
                .unwrap()
                .into_response()
        });

    let mock_objects_service = MockObjectsServiceInterface::new();
    let mock_thumbnails_service = MockThumbnailsServiceInterface::new();

    let app = Engine::new(mock_graphql_service, mock_objects_service, mock_thumbnails_service).into_inner();
    let actual = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/")
                .body(Body::empty())
                .unwrap())
        .await
        .unwrap();

    assert_eq!(actual.status(), 200);

    let expected = indoc! {r#"
        <!DOCTYPE html>
        <html lang="en">
            <meta charset="utf-8">
            <title>GraphiQL IDE</title>
        </html>
    "#};

    let actual = body::to_bytes(actual.into_body(), usize::MAX).await.unwrap();
    let actual = String::from_utf8(actual.to_vec()).unwrap();
    assert_eq!(actual, expected.to_string());
}
