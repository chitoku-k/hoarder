use std::{fs::File, io::{Read, Seek, SeekFrom, Write}, sync::Arc};

use application::service::media::MediaURLFactoryInterface;
use async_graphql::{value, EmptySubscription, Request, Schema, UploadValue, Variables};
use chrono::{TimeZone, Utc};
use domain::{
    entity::{
        media::MediumId,
        objects::{EntryUrl, EntryUrlPath},
        replicas::{Replica, ReplicaId, ReplicaStatus},
    },
    service::media::{MediumOverwriteBehavior, MediumSource},
};
use futures::future::ok;
use graphql::{mutation::Mutation, query::Query};
use indoc::indoc;
use pretty_assertions::assert_eq;
use serde_json::json;
use tempfile::tempfile;
use uuid::uuid;

mod mocks;
use mocks::{
    application::service::media::MockMediaURLFactoryInterface,
    domain::service::{
        external_services::MockExternalServicesServiceInterface,
        media::MockMediaServiceInterface,
        tags::MockTagsServiceInterface,
    },
    normalizer::MockNormalizerInterface,
};

#[tokio::test]
async fn succeeds_with_original_url() {
    let external_services_service = MockExternalServicesServiceInterface::new();
    let tags_service = MockTagsServiceInterface::new();
    let normalizer = MockNormalizerInterface::new();

    let mut media_service = MockMediaServiceInterface::new();
    media_service
        .expect_create_replica::<File>()
        .times(1)
        .withf(|medium_id, medium_source| {
            medium_id == &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")) &&
            matches!(medium_source, MediumSource::Url(url) if url == &EntryUrl::from("file:///77777777-7777-7777-7777-777777777777.png".to_string()))
        })
        .returning(|_, _| {
            Box::pin(ok(Replica {
                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                display_order: 1,
                thumbnail: None,
                original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
                mime_type: None,
                size: None,
                status: ReplicaStatus::Processing,
                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
            }))
        });

    let mut media_url_factory = MockMediaURLFactoryInterface::new();
    media_url_factory
        .expect_public_url()
        .times(1)
        .withf(|original_url| original_url == "file:///77777777-7777-7777-7777-777777777777.png")
        .returning(|_| Some("https://original.example.com/77777777-7777-7777-7777-777777777777.png".to_string()));

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let mutation = Mutation::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, mutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .data(Arc::new(normalizer))
        .data::<Arc<dyn MediaURLFactoryInterface>>(Arc::new(media_url_factory))
        .finish();

    let req = indoc! {r#"
        mutation {
            createReplica(
                mediumId: "77777777-7777-7777-7777-777777777777",
                originalUrl: "file:///77777777-7777-7777-7777-777777777777.png",
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
            "thumbnail": null,
            "url": "https://original.example.com/77777777-7777-7777-7777-777777777777.png",
            "originalUrl": "file:///77777777-7777-7777-7777-777777777777.png",
            "mimeType": null,
            "width": null,
            "height": null,
            "status": {
                "phase": "PROCESSING",
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
            Box::pin(ok(Replica {
                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                display_order: 1,
                thumbnail: None,
                original_url: "file:///77777777-7777-7777-7777-777777777777.png".to_string(),
                mime_type: None,
                size: None,
                status: ReplicaStatus::Processing,
                created_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 2, 0, 1, 0).unwrap(),
            }))
        });

    let mut media_url_factory = MockMediaURLFactoryInterface::new();
    media_url_factory
        .expect_public_url()
        .times(1)
        .withf(|original_url| original_url == "file:///77777777-7777-7777-7777-777777777777.png")
        .returning(|_| Some("https://original.example.com/77777777-7777-7777-7777-777777777777.png".to_string()));

    let query = Query::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let mutation = Mutation::<MockExternalServicesServiceInterface, MockMediaServiceInterface, MockTagsServiceInterface, MockNormalizerInterface>::new();
    let schema = Schema::build(query, mutation, EmptySubscription)
        .data(external_services_service)
        .data(media_service)
        .data(tags_service)
        .data(Arc::new(normalizer))
        .data::<Arc<dyn MediaURLFactoryInterface>>(Arc::new(media_url_factory))
        .finish();

    let query = indoc! {r#"
        mutation($file: Upload!) {
            createReplica(
                mediumId: "77777777-7777-7777-7777-777777777777",
                upload: {
                    file: $file,
                    overwrite: false,
                },
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
            "thumbnail": null,
            "url": "https://original.example.com/77777777-7777-7777-7777-777777777777.png",
            "originalUrl": "file:///77777777-7777-7777-7777-777777777777.png",
            "mimeType": null,
            "width": null,
            "height": null,
            "status": {
                "phase": "PROCESSING",
            },
            "createdAt": "2022-06-02T00:00:00+00:00",
            "updatedAt": "2022-06-02T00:01:00+00:00",
        },
    }));
}
