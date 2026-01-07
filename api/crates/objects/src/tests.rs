use std::sync::Arc;

use application::service::objects::ObjectsServiceInterface;
use axum::{body, http::header::{CONTENT_LENGTH, CONTENT_TYPE, LOCATION}};
use chrono::{TimeZone, Utc};
use domain::{entity::objects::{Entry, EntryKind, EntryMetadata, EntryUrl}, error::{Error, ErrorKind}};
use futures::future::{err, ok};
use pretty_assertions::assert_eq;

use crate::{ObjectsService, tests::mocks::{
    application::service::media::MockMediaURLFactoryInterface,
    domain::service::media::MockMediaServiceInterface,
}};

mod mocks;

#[tokio::test]
async fn serve_redirects_with_public_url() {
    let mut mock_media_service = MockMediaServiceInterface::new();
    mock_media_service
        .expect_get_object()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()))
        .returning(|_| {
            Box::pin(ok(Entry::new(
                "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
                Some(EntryUrl::from("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string())),
                EntryKind::Object,
                Some(EntryMetadata::new(
                    4096,
                    Some(Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap()),
                    Some(Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 1).unwrap()),
                    Some(Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 2).unwrap()),
                )),
            )))
        });

    let mut mock_media_url_factory = MockMediaURLFactoryInterface::new();
    mock_media_url_factory
        .expect_public_url()
        .times(1)
        .withf(|original_url| original_url == "file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg")
        .returning(|_| Some("https://original.example.com/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()));

    let objects_service = ObjectsService::new(mock_media_service, Arc::new(mock_media_url_factory));
    let actual = objects_service.serve("file:///aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string()).await;

    assert_eq!(actual.status(), 302);
    assert_eq!(actual.headers()[LOCATION], "https://original.example.com/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg");
}

#[tokio::test]
async fn serve_serves_content_without_public_url() {
    let mut mock_media_service = MockMediaServiceInterface::new();
    mock_media_service
        .expect_get_object()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string()))
        .returning(|_| {
            Box::pin(ok(Entry::new(
                "77777777-7777-7777-7777-777777777777.jpg".to_string(),
                Some(EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string())),
                EntryKind::Object,
                Some(EntryMetadata::new(
                    10000,
                    Some(Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap()),
                    Some(Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 1).unwrap()),
                    Some(Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 2).unwrap()),
                )),
            )))
        });

    mock_media_service
        .expect_read_object()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string()))
        .returning(|_| Box::pin(ok(&[0x01; 10000][..])));

    let mut mock_media_url_factory = MockMediaURLFactoryInterface::new();
    mock_media_url_factory
        .expect_public_url()
        .times(1)
        .withf(|original_url| original_url == "file:///77777777-7777-7777-7777-777777777777.png")
        .returning(|_| None);

    let objects_service = ObjectsService::new(mock_media_service, Arc::new(mock_media_url_factory));
    let actual = objects_service.serve("file:///77777777-7777-7777-7777-777777777777.png".to_string()).await;

    assert_eq!(actual.status(), 200);
    assert_eq!(actual.headers()[CONTENT_LENGTH], "10000");

    let actual = body::to_bytes(actual.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&actual, &[0x01; 10000][..]);
}

#[tokio::test]
async fn serve_fails_with_invalid_path() {
    let mut mock_media_service = MockMediaServiceInterface::new();
    mock_media_service
        .expect_get_object()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///%80.png".to_string()))
        .returning(|_| Box::pin(err(Error::from(ErrorKind::ObjectPathInvalid))));

    let mock_media_url_factory = MockMediaURLFactoryInterface::new();

    let objects_service = ObjectsService::new(mock_media_service, Arc::new(mock_media_url_factory));
    let actual = objects_service.serve("file:///%80.png".to_string()).await;

    assert_eq!(actual.status(), 400);
    assert_eq!(actual.headers()[CONTENT_TYPE], "text/plain; charset=utf-8");

    let actual = body::to_bytes(actual.into_body(), usize::MAX).await.unwrap();
    let actual = String::from_utf8(actual.to_vec()).unwrap();
    assert_eq!(actual, "Bad Request: object path invalid\n");
}

#[tokio::test]
async fn serve_fails_with_invalid_url() {
    let mut mock_media_service = MockMediaServiceInterface::new();
    mock_media_service
        .expect_get_object()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string()))
        .returning(|_| Box::pin(err(Error::from(ErrorKind::ObjectUrlInvalid { url: "file:///77777777-7777-7777-7777-777777777777.png".to_string() }))));

    let mock_media_url_factory = MockMediaURLFactoryInterface::new();

    let objects_service = ObjectsService::new(mock_media_service, Arc::new(mock_media_url_factory));
    let actual = objects_service.serve("file:///77777777-7777-7777-7777-777777777777.png".to_string()).await;

    assert_eq!(actual.status(), 400);
    assert_eq!(actual.headers()[CONTENT_TYPE], "text/plain; charset=utf-8");

    let actual = body::to_bytes(actual.into_body(), usize::MAX).await.unwrap();
    let actual = String::from_utf8(actual.to_vec()).unwrap();
    assert_eq!(actual, "Bad Request: object url invalid\n");
}

#[tokio::test]
async fn serve_fails_with_unsupported() {
    let mut mock_media_service = MockMediaServiceInterface::new();
    mock_media_service
        .expect_get_object()
        .times(1)
        .withf(|url| url == &EntryUrl::from("s3:///77777777-7777-7777-7777-777777777777.png".to_string()))
        .returning(|_| Box::pin(err(Error::from(ErrorKind::ObjectUrlUnsupported { url: "s3:///77777777-7777-7777-7777-777777777777.png".to_string() }))));

    let mock_media_url_factory = MockMediaURLFactoryInterface::new();

    let objects_service = ObjectsService::new(mock_media_service, Arc::new(mock_media_url_factory));
    let actual = objects_service.serve("s3:///77777777-7777-7777-7777-777777777777.png".to_string()).await;

    assert_eq!(actual.status(), 400);
    assert_eq!(actual.headers()[CONTENT_TYPE], "text/plain; charset=utf-8");

    let actual = body::to_bytes(actual.into_body(), usize::MAX).await.unwrap();
    let actual = String::from_utf8(actual.to_vec()).unwrap();
    assert_eq!(actual, "Bad Request: object url unsupported\n");
}

#[tokio::test]
async fn serve_fails_with_not_found() {
    let mut mock_media_service = MockMediaServiceInterface::new();
    mock_media_service
        .expect_get_object()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string()))
        .returning(|_| Box::pin(err(Error::from(ErrorKind::ObjectNotFound { url: "file:///77777777-7777-7777-7777-777777777777.png".to_string() }))));

    let mock_media_url_factory = MockMediaURLFactoryInterface::new();

    let objects_service = ObjectsService::new(mock_media_service, Arc::new(mock_media_url_factory));
    let actual = objects_service.serve("file:///77777777-7777-7777-7777-777777777777.png".to_string()).await;

    assert_eq!(actual.status(), 404);
    assert_eq!(actual.headers()[CONTENT_TYPE], "text/plain; charset=utf-8");

    let actual = body::to_bytes(actual.into_body(), usize::MAX).await.unwrap();
    let actual = String::from_utf8(actual.to_vec()).unwrap();
    assert_eq!(actual, "Not Found\n");
}

#[tokio::test]
async fn serve_fails_with_internal_server_error() {
    let mut mock_media_service = MockMediaServiceInterface::new();
    mock_media_service
        .expect_get_object()
        .times(1)
        .withf(|url| url == &EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string()))
        .returning(|_| Box::pin(err(Error::from(ErrorKind::ObjectReadFailed { url: "file:///77777777-7777-7777-7777-777777777777.png".to_string() }))));

    let mock_media_url_factory = MockMediaURLFactoryInterface::new();

    let objects_service = ObjectsService::new(mock_media_service, Arc::new(mock_media_url_factory));
    let actual = objects_service.serve("file:///77777777-7777-7777-7777-777777777777.png".to_string()).await;

    assert_eq!(actual.status(), 500);
    assert_eq!(actual.headers()[CONTENT_TYPE], "text/plain; charset=utf-8");

    let actual = body::to_bytes(actual.into_body(), usize::MAX).await.unwrap();
    let actual = String::from_utf8(actual.to_vec()).unwrap();
    assert_eq!(actual, "Internal Server Error\n");
}
