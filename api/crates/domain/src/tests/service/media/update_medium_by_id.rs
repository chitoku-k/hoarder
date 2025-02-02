use anyhow::anyhow;
use chrono::{TimeZone, Utc};
use futures::future::{err, ok};
use ordermap::OrderMap;
use pretty_assertions::{assert_eq, assert_matches};
use tokio_util::task::TaskTracker;
use uuid::uuid;

use crate::{
    entity::{
        media::{Medium, MediumId},
        replicas::ReplicaId,
        sources::SourceId,
        tag_types::TagTypeId,
        tags::{TagDepth, TagId},
    },
    error::{Error, ErrorKind},
    service::media::{MediaService, MediaServiceInterface},
};

use super::mocks::domain::{
    processor::media::MockMediumImageProcessor,
    repository::{
        media::MockMediaRepository,
        objects::MockObjectsRepository,
        replicas::MockReplicasRepository,
        sources::MockSourcesRepository,
    },
};

#[tokio::test]
async fn succeeds() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_update_by_id()
        .times(1)
        .withf(|
            id,
            add_source_ids,
            remove_source_ids,
            add_tag_tag_type_ids,
            remove_tag_tag_type_ids,
            replica_orders,
            created_at,
            tag_depth,
            replicas,
            sources,
        | {
            add_source_ids.clone_box().eq([
                SourceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            ]) &&
            remove_source_ids.clone_box().eq([
                SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            ]) &&
            add_tag_tag_type_ids.clone_box().eq([
                (
                    TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
            ]) &&
            remove_tag_tag_type_ids.clone_box().eq([
                (
                    TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
            ]) &&
            replica_orders.clone_box().eq([
                ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
            ]) &&
            (
                id,
                created_at,
                tag_depth,
                replicas,
                sources,
            ) == (
                &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                &Some(Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap()),
                &Some(TagDepth::new(1, 1)),
                &true,
                &true,
            )
        })
        .returning(|_, _, _, _, _, _, _, _, _, _| {
            Box::pin(ok(Medium {
                id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                sources: Vec::new(),
                tags: OrderMap::new(),
                replicas: Vec::new(),
                created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
            }))
        });

    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();
    let task_tracker = TaskTracker::new();

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor, task_tracker);
    let actual = service.update_medium_by_id(
        MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        [
            SourceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        ].into_iter(),
        [
            SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            ),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            ),
        ].into_iter(),
        [
            ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
            ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        ].into_iter(),
        Some(Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap()),
        Some(TagDepth::new(1, 1)),
        true,
        true,
    ).await.unwrap();

    assert_eq!(actual, Medium {
        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        sources: Vec::new(),
        tags: OrderMap::new(),
        replicas: Vec::new(),
        created_at: Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2022, 6, 1, 0, 5, 0).unwrap(),
    });
}

#[tokio::test]
async fn fails() {
    let mut mock_media_repository = MockMediaRepository::new();
    mock_media_repository
        .expect_update_by_id()
        .times(1)
        .withf(|
            id,
            add_source_ids,
            remove_source_ids,
            add_tag_tag_type_ids,
            remove_tag_tag_type_ids,
            replica_orders,
            created_at,
            tag_depth,
            replicas,
            sources,
        | {
            add_source_ids.clone_box().eq([
                SourceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            ]) &&
            remove_source_ids.clone_box().eq([
                SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            ]) &&
            add_tag_tag_type_ids.clone_box().eq([
                (
                    TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
            ]) &&
            remove_tag_tag_type_ids.clone_box().eq([
                (
                    TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
            ]) &&
            replica_orders.clone_box().eq([
                ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
            ]) &&
            (
                id,
                created_at,
                tag_depth,
                replicas,
                sources,
            ) == (
                &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                &Some(Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap()),
                &Some(TagDepth::new(1, 1)),
                &true,
                &true,
            )
        })
        .returning(|_, _, _, _, _, _, _, _, _, _| Box::pin(err(Error::other(anyhow!("error communicating with database")))));

    let mock_objects_repository = MockObjectsRepository::new();
    let mock_replicas_repository = MockReplicasRepository::new();
    let mock_sources_repository = MockSourcesRepository::new();
    let mock_medium_image_processor = MockMediumImageProcessor::new();
    let task_tracker = TaskTracker::new();

    let service = MediaService::new(mock_media_repository, mock_objects_repository, mock_replicas_repository, mock_sources_repository, mock_medium_image_processor, task_tracker);
    let actual = service.update_medium_by_id(
        MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        [
            SourceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
        ].into_iter(),
        [
            SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            ),
        ].into_iter(),
        [
            (
                TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
            ),
        ].into_iter(),
        [
            ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
            ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
        ].into_iter(),
        Some(Utc.with_ymd_and_hms(2022, 6, 1, 12, 34, 56).unwrap()),
        Some(TagDepth::new(1, 1)),
        true,
        true,
    ).await.unwrap_err();

    assert_matches!(actual.kind(), ErrorKind::Other);
}
