use application::{
    server::Engine,
    service::{
        graphql::MockGraphQLServiceInterface,
        objects::MockObjectsServiceInterface,
        thumbnails::MockThumbnailsServiceInterface,
    },
};
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
        .expect_endpoint()
        .times(1)
        .return_const("/graphql".to_string());

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
        .expect_endpoint()
        .times(1)
        .return_const("/graphql".to_string());

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

#[tokio::test]
async fn thumbnail_show() {
    let expected = vec![0x01, 0x02, 0x03, 0x04];

    let mut mock_graphql_service = MockGraphQLServiceInterface::new();
    mock_graphql_service
        .expect_endpoint()
        .times(1)
        .return_const("/graphql".to_string());

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
