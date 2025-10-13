use std::{fs::File, io::{Read, Seek, SeekFrom, Write}, sync::Arc};

use application::service::{media::MediaURLFactoryInterface, thumbnails::ThumbnailURLFactoryInterface};
use async_graphql::{value, EmptySubscription, Request, Schema, UploadValue, Variables};
use chrono::{TimeZone, Utc};
use domain::{
    entity::{
        media::MediumId,
        objects::{EntryUrl, EntryUrlPath},
        replicas::{Replica, ReplicaId, ReplicaStatus, Size, Thumbnail, ThumbnailId},
    },
    service::media::{MediumOverwriteBehavior, MediumSource},
};
use futures::{future::ok, FutureExt};
use indoc::indoc;
use pretty_assertions::assert_eq;
use serde_json::json;
use tempfile::tempfile;
use tokio_util::task::TaskTracker;
use uuid::uuid;

use crate::{mutation::Mutation, query::Query, tests::mocks::application::service::thumbnails::MockThumbnailURLFactoryInterface};

use super::mocks::{
    application::service::media::MockMediaURLFactoryInterface,
    domain::service::{
        external_services::MockExternalServicesServiceInterface,
        media::MockMediaServiceInterface,
        tags::MockTagsServiceInterface,
    },
    normalizer::MockNormalizerInterface,
    query::MockQueryParserInterface,
};

#[tokio::test]
async fn succeeds_with_original_url() {
    let external_services_service = MockExternalServicesServiceInterface::new();
    let tags_service = MockTagsServiceInterface::new();
    let normalizer = MockNormalizerInterface::new();
    let task_tracker = TaskTracker::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_create_replica::<File>()
        .times(1)
        .withf(|medium_id, medium_source| {
            medium_id == &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")) &&
            matches!(medium_source, MediumSource::Url(url) if url == &EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string()))
        })
        .returning(|_, _| {
            Box::pin(ok((
                Replica {
                    id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                    display_order: 1,
                    thumbnail: None,
                    original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
                    mime_type: None,
                    size: None,
                    status: ReplicaStatus::Processing,
                    created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
                },
                ok(Replica {
                    id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                    display_order: 1,
                    thumbnail: Some(Thumbnail {
                        id: ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                        size: Size::new(240, 240),
                        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 2, 0).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 3, 0).unwrap(),
                    }),
                    original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
                    mime_type: Some("image/png".to_string()),
                    size: Some(Size::new(720, 720)),
                    status: ReplicaStatus::Ready,
                    created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
                }).boxed(),
            )))
        });

    let mut media_url_factory = MockMediaURLFactoryInterface::new();
    media_url_factory
        .expect_public_url()
        .times(1)
        .withf(|original_url| original_url == "file:///77777777-7777-7777-7777-777777777777.png")
        .returning(|_| Some("https://original.example.com/77777777-7777-7777-7777-777777777777.png".to_string()));

    let mut thumbnail_url_factory = MockThumbnailURLFactoryInterface::new();
    thumbnail_url_factory
        .expect_get()
        .times(1)
        .withf(|thumbnail_id| thumbnail_id == &ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888")))
        .returning(|_| "https://img.example.com/88888888-8888-8888-8888-888888888888".to_string());

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface, MockQueryParserInterface>::new();
    let mutation = Mutation::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, mutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .data(normalizer)
        .data::<Arc<dyn MediaURLFactoryInterface>>(Arc::new(media_url_factory))
        .data::<Arc<dyn ThumbnailURLFactoryInterface>>(Arc::new(thumbnail_url_factory))
        .data(task_tracker)
        .finish();

    let req = indoc! {r#"
        mutation {
            createReplica(
                mediumId: "77777777-7777-7777-7777-777777777777",
                originalUrl: "file:///77777777-7777-7777-7777-777777777777.png",
                sync: true,
            ) {
                id
                displayOrder
                thumbnail {
                    id
                    url
                    width
                    height
                    createdAt
                    updatedAt
                }
                url
                originalUrl
                mimeType
                width
                height
                status {
                    phase
                }
                createdAt
                updatedAt
            }
        }
    "#};
    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "createReplica": {
            "id": "66666666-6666-6666-6666-666666666666",
            "displayOrder": 1,
            "thumbnail": {
                "id": "88888888-8888-8888-8888-888888888888",
                "url": "https://img.example.com/88888888-8888-8888-8888-888888888888",
                "width": 240,
                "height": 240,
                "createdAt": "2022-06-02T00:02:00+00:00",
                "updatedAt": "2022-06-02T00:03:00+00:00",
            },
            "url": "https://original.example.com/77777777-7777-7777-7777-777777777777.png",
            "originalUrl": "file:///77777777-7777-7777-7777-777777777777.png",
            "mimeType": "image/png",
            "width": 720,
            "height": 720,
            "status": {
                "phase": "READY",
            },
            "createdAt": "2022-06-02T00:00:00+00:00",
            "updatedAt": "2022-06-02T00:01:00+00:00",
        },
    }));
}

#[tokio::test]
async fn succeeds_with_upload() {
    let external_services_service = MockExternalServicesServiceInterface::new();
    let tags_service = MockTagsServiceInterface::new();
    let normalizer = MockNormalizerInterface::new();
    let task_tracker = TaskTracker::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_create_replica::<File>()
        .times(1)
        .withf(|medium_id, medium_source| {
            medium_id == &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")) &&
            matches!(medium_source, MediumSource::Content(path, read, overwrite) if (path, overwrite) == (
                &EntryUrlPath::from("/77777777-7777-7777-7777-777777777777.png".to_string()),
                &MediumOverwriteBehavior::Fail,
            ) && {
                let mut buf = Vec::with_capacity(8);
                let mut file = read.try_clone().unwrap();
                file.read_to_end(&mut buf).unwrap();
                file.seek(SeekFrom::Start(0)).unwrap();
                buf == [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]
            })
        })
        .returning(|_, _| {
            Box::pin(ok((
                Replica {
                    id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                    display_order: 1,
                    thumbnail: None,
                    original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
                    mime_type: None,
                    size: None,
                    status: ReplicaStatus::Processing,
                    created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
                },
                ok(Replica {
                    id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                    display_order: 1,
                    thumbnail: Some(Thumbnail {
                        id: ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                        size: Size::new(240, 240),
                        created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 2, 0).unwrap(),
                        updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 3, 0).unwrap(),
                    }),
                    original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
                    mime_type: Some("image/png".to_string()),
                    size: Some(Size::new(720, 720)),
                    status: ReplicaStatus::Ready,
                    created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
                }).boxed(),
            )))
        });

    let mut media_url_factory = MockMediaURLFactoryInterface::new();
    media_url_factory
        .expect_public_url()
        .times(1)
        .withf(|original_url| original_url == "file:///77777777-7777-7777-7777-777777777777.png")
        .returning(|_| Some("https://original.example.com/77777777-7777-7777-7777-777777777777.png".to_string()));

    let mut thumbnail_url_factory = MockThumbnailURLFactoryInterface::new();
    thumbnail_url_factory
        .expect_get()
        .times(1)
        .withf(|thumbnail_id| thumbnail_id == &ThumbnailId::from(uuid!("88888888-8888-8888-8888-888888888888")))
        .returning(|_| "https://img.example.com/88888888-8888-8888-8888-888888888888".to_string());

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface, MockQueryParserInterface>::new();
    let mutation = Mutation::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, mutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .data(normalizer)
        .data::<Arc<dyn MediaURLFactoryInterface>>(Arc::new(media_url_factory))
        .data::<Arc<dyn ThumbnailURLFactoryInterface>>(Arc::new(thumbnail_url_factory))
        .data(task_tracker)
        .finish();

    let query = indoc! {r#"
        mutation($file: Upload!) {
            createReplica(
                mediumId: "77777777-7777-7777-7777-777777777777",
                upload: {
                    file: $file,
                    overwrite: false,
                },
                sync: true,
            ) {
                id
                displayOrder
                thumbnail {
                    id
                    url
                    width
                    height
                    createdAt
                    updatedAt
                }
                url
                originalUrl
                mimeType
                width
                height
                status {
                    phase
                }
                createdAt
                updatedAt
            }
        }
    "#};

    let mut file = tempfile().unwrap();
    file.write_all(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();

    let mut req = Request::new(query).variables(Variables::from_json(json!({"file": null})));
    req.set_upload("variables.file", UploadValue {
        filename: "/77777777-7777-7777-7777-777777777777.png".to_string(),
        content_type: Some("image/png".to_string()),
        content: file,
    });

    let actual = schema.execute(req).await.into_result().unwrap();

    assert_eq!(actual.data, value!({
        "createReplica": {
            "id": "66666666-6666-6666-6666-666666666666",
            "displayOrder": 1,
            "thumbnail": {
                "id": "88888888-8888-8888-8888-888888888888",
                "url": "https://img.example.com/88888888-8888-8888-8888-888888888888",
                "width": 240,
                "height": 240,
                "createdAt": "2022-06-02T00:02:00+00:00",
                "updatedAt": "2022-06-02T00:03:00+00:00",
            },
            "url": "https://original.example.com/77777777-7777-7777-7777-777777777777.png",
            "originalUrl": "file:///77777777-7777-7777-7777-777777777777.png",
            "mimeType": "image/png",
            "width": 720,
            "height": 720,
            "status": {
                "phase": "READY",
            },
            "createdAt": "2022-06-02T00:00:00+00:00",
            "updatedAt": "2022-06-02T00:01:00+00:00",
        },
    }));
}
