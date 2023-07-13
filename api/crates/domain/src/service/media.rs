use async_trait::async_trait;
use chrono::NaiveDateTime;
use derive_more::Constructor;

use crate::{
    entity::{
        external_services::{ExternalMetadata, ExternalServiceId},
        media::{Medium, MediumId},
        replicas::{Replica, ReplicaId, ReplicaThumbnail},
        sources::{Source, SourceId},
        tag_types::TagTypeId,
        tags::{TagDepth, TagId},
    },
    repository::{media, replicas, sources, DeleteResult, OrderDirection},
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MediaServiceInterface: Send + Sync + 'static {
    /// Creates a medium.
    async fn create_medium<T, U>(&self, source_ids: T, created_at: Option<NaiveDateTime>, tag_tag_type_ids: U, tag_depth: Option<TagDepth>, sources: bool) -> anyhow::Result<Medium>
    where
        T: IntoIterator<Item = SourceId> + Send + Sync + 'static,
        U: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static;

    /// Creates a replica.
    async fn create_replica(&self, medium_id: MediumId, thumbnail: Option<Vec<u8>>, original_url: &str, mime_type: &str) -> anyhow::Result<Replica>;

    /// Creates a source.
    async fn create_source(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> anyhow::Result<Source>;

    /// Gets media.
    async fn get_media(
        &self,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
        since: Option<(NaiveDateTime, MediumId)>,
        until: Option<(NaiveDateTime, MediumId)>,
        order: OrderDirection,
        limit: u64,
    ) -> anyhow::Result<Vec<Medium>>;

    /// Gets the media by their IDs.
    async fn get_media_by_ids<T>(&self, ids: T, tag_depth: Option<TagDepth>, replicas: bool, sources: bool) -> anyhow::Result<Vec<Medium>>
    where
        T: IntoIterator<Item = MediumId> + Send + Sync + 'static;

    /// Gets the media by their source IDs.
    async fn get_media_by_source_ids<T>(
        &self,
        source_ids: T,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
        since: Option<(NaiveDateTime, MediumId)>,
        until: Option<(NaiveDateTime, MediumId)>,
        order: OrderDirection,
        limit: u64,
    ) -> anyhow::Result<Vec<Medium>>
    where
        T: IntoIterator<Item = SourceId> + Send + Sync + 'static;

    /// Gets the media by their tag IDs.
    async fn get_media_by_tag_ids<T>(
        &self,
        tag_tag_type_ids: T,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
        since: Option<(NaiveDateTime, MediumId)>,
        until: Option<(NaiveDateTime, MediumId)>,
        order: OrderDirection,
        limit: u64,
    ) -> anyhow::Result<Vec<Medium>>
    where
        T: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static;

    /// Gets the replicas by their IDs.
    async fn get_replicas_by_ids<T>(&self, ids: T) -> anyhow::Result<Vec<Replica>>
    where
        T: IntoIterator<Item = ReplicaId> + Send + Sync + 'static;

    /// Gets the replica by original URL.
    async fn get_replica_by_original_url(&self, original_url: &str) -> anyhow::Result<Replica>;

    /// Gets the source by its external metadata.
    async fn get_source_by_external_metadata(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> anyhow::Result<Source>;

    /// Gets the thumbnail by ID.
    async fn get_thumbnail_by_id(&self, id: ReplicaId) -> anyhow::Result<ReplicaThumbnail>;

    /// Updates the medium by ID.
    async fn update_medium_by_id<T, U, V, W, X>(
        &self,
        id: MediumId,
        add_source_ids: T,
        remove_source_ids: U,
        add_tag_tag_type_ids: V,
        remove_tag_tag_type_ids: W,
        replica_orders: X,
        created_at: Option<NaiveDateTime>,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
    ) -> anyhow::Result<Medium>
    where
        T: IntoIterator<Item = SourceId> + Send + Sync + 'static,
        U: IntoIterator<Item = SourceId> + Send + Sync + 'static,
        V: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
        W: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
        X: IntoIterator<Item = ReplicaId> + Send + Sync + 'static;

    /// Updates the replica by ID.
    async fn update_replica_by_id<'a, 'b>(&self, id: ReplicaId, thumbnail: Option<Vec<u8>>, original_url: Option<&'a str>, mime_type: Option<&'b str>) -> anyhow::Result<Replica>;

    /// Updates the source by ID.
    async fn update_source_by_id(&self, id: SourceId, external_service_id: Option<ExternalServiceId>, external_metadata: Option<ExternalMetadata>) -> anyhow::Result<Source>;

    /// Deletes the medium by ID.
    async fn delete_medium_by_id(&self, id: MediumId) -> anyhow::Result<DeleteResult>;

    /// Deletes the replica by ID.
    async fn delete_replica_by_id(&self, id: ReplicaId) -> anyhow::Result<DeleteResult>;

    /// Deletes the source by ID.
    async fn delete_source_by_id(&self, id: SourceId) -> anyhow::Result<DeleteResult>;
}

#[derive(Clone, Constructor)]
pub struct MediaService<MediaRepository, ReplicasRepository, SourcesRepository> {
    media_repository: MediaRepository,
    replicas_repository: ReplicasRepository,
    sources_repository: SourcesRepository,
}

#[async_trait]
impl<MediaRepository, ReplicasRepository, SourcesRepository> MediaServiceInterface for MediaService<MediaRepository, ReplicasRepository, SourcesRepository>
where
    MediaRepository: media::MediaRepository,
    ReplicasRepository: replicas::ReplicasRepository,
    SourcesRepository: sources::SourcesRepository,
{
    async fn create_medium<T, U>(&self, source_ids: T, created_at: Option<NaiveDateTime>, tag_tag_type_ids: U, tag_depth: Option<TagDepth>, sources: bool) -> anyhow::Result<Medium>
    where
        T: IntoIterator<Item = SourceId> + Send + Sync + 'static,
        U: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
    {
        match self.media_repository.create(source_ids, created_at, tag_tag_type_ids, tag_depth, sources).await {
            Ok(medium) => Ok(medium),
            Err(e) => {
                log::error!("failed to create a medium\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn create_replica(&self, medium_id: MediumId, thumbnail: Option<Vec<u8>>, original_url: &str, mime_type: &str) -> anyhow::Result<Replica> {
        match self.replicas_repository.create(medium_id, thumbnail, original_url, mime_type).await {
            Ok(replica) => Ok(replica),
            Err(e) => {
                log::error!("failed to create a replica\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn create_source(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> anyhow::Result<Source> {
        match self.sources_repository.create(external_service_id, external_metadata).await {
            Ok(source) => Ok(source),
            Err(e) => {
                log::error!("failed to create a source\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_media(
        &self,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
        since: Option<(NaiveDateTime, MediumId)>,
        until: Option<(NaiveDateTime, MediumId)>,
        order: OrderDirection,
        limit: u64,
    ) -> anyhow::Result<Vec<Medium>> {
        match self.media_repository.fetch_all(tag_depth, replicas, sources, since, until, order, limit).await {
            Ok(media) => Ok(media),
            Err(e) => {
                log::error!("failed to get the media\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_media_by_ids<T>(&self, ids: T, tag_depth: Option<TagDepth>, replicas: bool, sources: bool) -> anyhow::Result<Vec<Medium>>
    where
        T: IntoIterator<Item = MediumId> + Send + Sync + 'static,
    {
        match self.media_repository.fetch_by_ids(ids, tag_depth, replicas, sources).await {
            Ok(media) => Ok(media),
            Err(e) => {
                log::error!("failed to get the media\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_media_by_source_ids<T>(
        &self,
        source_ids: T,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
        since: Option<(NaiveDateTime, MediumId)>,
        until: Option<(NaiveDateTime, MediumId)>,
        order: OrderDirection,
        limit: u64,
    ) -> anyhow::Result<Vec<Medium>>
    where
        T: IntoIterator<Item = SourceId> + Send + Sync + 'static,
    {
        match self.media_repository.fetch_by_source_ids(source_ids, tag_depth, replicas, sources, since, until, order, limit).await {
            Ok(media) => Ok(media),
            Err(e) => {
                log::error!("failed to get the media\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_media_by_tag_ids<T>(
        &self,
        tag_tag_type_ids: T,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
        since: Option<(NaiveDateTime, MediumId)>,
        until: Option<(NaiveDateTime, MediumId)>,
        order: OrderDirection,
        limit: u64,
    ) -> anyhow::Result<Vec<Medium>>
    where
        T: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
    {
        match self.media_repository.fetch_by_tag_ids(tag_tag_type_ids, tag_depth, replicas, sources, since, until, order, limit).await {
            Ok(media) => Ok(media),
            Err(e) => {
                log::error!("failed to get the media\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_replicas_by_ids<T>(&self, ids: T) -> anyhow::Result<Vec<Replica>>
    where
        T: IntoIterator<Item = ReplicaId> + Send + Sync + 'static,
    {
        match self.replicas_repository.fetch_by_ids(ids).await {
            Ok(replicas) => Ok(replicas),
            Err(e) => {
                log::error!("failed to get the replicas\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_replica_by_original_url(&self, original_url: &str) -> anyhow::Result<Replica> {
        match self.replicas_repository.fetch_by_original_url(original_url).await {
            Ok(replica) => Ok(replica),
            Err(e) => {
                log::error!("failed to get the replica\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_source_by_external_metadata(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> anyhow::Result<Source> {
        match self.sources_repository.fetch_by_external_metadata(external_service_id, external_metadata).await {
            Ok(source) => Ok(source),
            Err(e) => {
                log::error!("failed to get the source\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_thumbnail_by_id(&self, id: ReplicaId) -> anyhow::Result<ReplicaThumbnail> {
        match self.replicas_repository.fetch_thumbnail_by_id(id).await {
            Ok(replica) => Ok(replica),
            Err(e) => {
                log::error!("failed to get the replica\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn update_medium_by_id<T, U, V, W, X>(
        &self,
        id: MediumId,
        add_source_ids: T,
        remove_source_ids: U,
        add_tag_tag_type_ids: V,
        remove_tag_tag_type_ids: W,
        replica_orders: X,
        created_at: Option<NaiveDateTime>,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
    ) -> anyhow::Result<Medium>
    where
        T: IntoIterator<Item = SourceId> + Send + Sync + 'static,
        U: IntoIterator<Item = SourceId> + Send + Sync + 'static,
        V: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
        W: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
        X: IntoIterator<Item = ReplicaId> + Send + Sync + 'static,
    {
        match self.media_repository.update_by_id(id, add_source_ids, remove_source_ids, add_tag_tag_type_ids, remove_tag_tag_type_ids, replica_orders, created_at, tag_depth, replicas, sources).await {
            Ok(medium) => Ok(medium),
            Err(e) => {
                log::error!("failed to update the medium\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn update_replica_by_id<'a, 'b>(&self, id: ReplicaId, thumbnail: Option<Vec<u8>>, original_url: Option<&'a str>, mime_type: Option<&'b str>) -> anyhow::Result<Replica> {
        match self.replicas_repository.update_by_id(id, thumbnail, original_url, mime_type).await {
            Ok(replica) => Ok(replica),
            Err(e) => {
                log::error!("failed to update the replica\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn update_source_by_id(&self, id: SourceId, external_service_id: Option<ExternalServiceId>, external_metadata: Option<ExternalMetadata>) -> anyhow::Result<Source> {
        match self.sources_repository.update_by_id(id, external_service_id, external_metadata).await {
            Ok(source) => Ok(source),
            Err(e) => {
                log::error!("failed to update the source\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn delete_medium_by_id(&self, id: MediumId) -> anyhow::Result<DeleteResult> {
        match self.media_repository.delete_by_id(id).await {
            Ok(result) => Ok(result),
            Err(e) => {
                log::error!("failed to delete the medium\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn delete_replica_by_id(&self, id: ReplicaId) -> anyhow::Result<DeleteResult> {
        match self.replicas_repository.delete_by_id(id).await {
            Ok(result) => Ok(result),
            Err(e) => {
                log::error!("failed to delete the replica\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn delete_source_by_id(&self, id: SourceId) -> anyhow::Result<DeleteResult> {
        match self.sources_repository.delete_by_id(id).await {
            Ok(result) => Ok(result),
            Err(e) => {
                log::error!("failed to delete the source\nError: {e:?}");
                Err(e)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use anyhow::anyhow;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use uuid::uuid;

    use crate::{
        entity::{
            external_services::{ExternalMetadata, ExternalService},
            tag_types::{TagType, TagTypeId},
            tags::{Tag, TagId},
        },
        repository::{
            media::MockMediaRepository,
            replicas::MockReplicasRepository,
            sources::MockSourcesRepository,
            OrderDirection,
        },
    };

    use super::*;

    #[tokio::test]
    async fn create_medium_succeeds() {
        let mut mock_media_repository = MockMediaRepository::new();
        mock_media_repository
            .expect_create()
            .times(1)
            .withf(|source_ids, created_at, tag_tag_type_ids, tag_depth, sources| {
                (source_ids, created_at, tag_tag_type_ids, tag_depth, sources) == (
                    &vec![
                        SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                        SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    ],
                    &Some(NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap()),
                    &vec![
                        (
                            TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                            TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                        ),
                        (
                            TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                            TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                        ),
                    ],
                    &None,
                    &true,
                )
            })
            .returning(|_, _, _, _, _| {
                Ok(Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: vec![
                        Source {
                            id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                            external_service: ExternalService {
                                id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                slug: "twitter".to_string(),
                                name: "Twitter".to_string(),
                            },
                            external_metadata: ExternalMetadata::Twitter { id: 727620202049900544 },
                            created_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 1)).unwrap(),
                        },
                        Source {
                            id: SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                            external_service: ExternalService {
                                id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                                slug: "pixiv".to_string(),
                                name: "pixiv".to_string(),
                            },
                            external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
                            created_at: NaiveDate::from_ymd_opt(2016, 5, 6).and_then(|d| d.and_hms_opt(5, 14, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2016, 5, 6).and_then(|d| d.and_hms_opt(5, 14, 1)).unwrap(),
                        },
                    ],
                    tags: {
                        let mut tags = BTreeMap::new();
                        tags.insert(
                            TagType {
                                id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                                slug: "character".to_string(),
                                name: "キャラクター".to_string(),
                            },
                            vec![
                                Tag {
                                    id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                                    name: "赤座あかり".to_string(),
                                    kana: "あかざあかり".to_string(),
                                    aliases: Default::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                                },
                            ],
                        );
                        tags.insert(
                            TagType {
                                id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                                slug: "character".to_string(),
                                name: "キャラクター".to_string(),
                            },
                            vec![
                                Tag {
                                    id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                                    name: "歳納京子".to_string(),
                                    kana: "としのうきょうこ".to_string(),
                                    aliases: Default::default(),
                                    parent: None,
                                    children: Vec::new(),
                                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                                },
                            ],
                        );
                        tags
                    },
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                })
            });

        let mock_replicas_repository = MockReplicasRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.create_medium(
            vec![
                SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            ],
            Some(NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap()),
            vec![
                (
                    TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
                (
                    TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
            ],
            None,
            true,
        ).await.unwrap();

        assert_eq!(actual, Medium {
            id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
            sources: vec![
                Source {
                    id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                        slug: "twitter".to_string(),
                        name: "Twitter".to_string(),
                    },
                    external_metadata: ExternalMetadata::Twitter { id: 727620202049900544 },
                    created_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 1)).unwrap(),
                },
                Source {
                    id: SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                        slug: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
                    created_at: NaiveDate::from_ymd_opt(2016, 5, 6).and_then(|d| d.and_hms_opt(5, 14, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2016, 5, 6).and_then(|d| d.and_hms_opt(5, 14, 1)).unwrap(),
                },
            ],
            tags: {
                let mut tags = BTreeMap::new();
                tags.insert(
                    TagType {
                        id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                        slug: "character".to_string(),
                        name: "キャラクター".to_string(),
                    },
                    vec![
                        Tag {
                            id: TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                            name: "赤座あかり".to_string(),
                            kana: "あかざあかり".to_string(),
                            aliases: Default::default(),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                        },
                    ],
                );
                tags.insert(
                    TagType {
                        id: TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                        slug: "character".to_string(),
                        name: "キャラクター".to_string(),
                    },
                    vec![
                        Tag {
                            id: TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                            name: "歳納京子".to_string(),
                            kana: "としのうきょうこ".to_string(),
                            aliases: Default::default(),
                            parent: None,
                            children: Vec::new(),
                            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                        },
                    ],
                );
                tags
            },
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
        });
    }

    #[tokio::test]
    async fn create_medium_fails() {
        let mut mock_media_repository = MockMediaRepository::new();
        mock_media_repository
            .expect_create()
            .times(1)
            .withf(|source_ids, created_at, tag_tag_type_ids, tag_depth, sources| {
                (source_ids, created_at, tag_tag_type_ids, tag_depth, sources) == (
                    &vec![
                        SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                        SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    ],
                    &Some(NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap()),
                    &vec![
                        (
                            TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                            TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                        ),
                        (
                            TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                            TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                        ),
                    ],
                    &None,
                    &true,
                )
            })
            .returning(|_, _, _, _, _| Err(anyhow!("error creating a medium")));

        let mock_replicas_repository = MockReplicasRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.create_medium(
            vec![
                SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            ],
            Some(NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap()),
            vec![
                (
                    TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
                (
                    TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
            ],
            None,
            true,
        ).await;

        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn create_replica_succeeds() {
        let mock_media_repository = MockMediaRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let mut mock_replicas_repository = MockReplicasRepository::new();
        mock_replicas_repository
            .expect_create()
            .times(1)
            .withf(|medium_id, thumbnail, original_url, mime_type| {
                (medium_id, thumbnail, original_url, mime_type) == (
                    &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    &Some(vec![0x01, 0x02, 0x03, 0x04]),
                    "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png",
                    "image/png",
                )
            })
            .returning(|_, _, _, _| {
                Ok(Replica {
                    id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                    display_order: Some(1),
                    has_thumbnail: true,
                    original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                })
            });

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.create_replica(
            MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
            Some(vec![0x01, 0x02, 0x03, 0x04]),
            "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png",
            "image/png",
        ).await.unwrap();

        assert_eq!(actual, Replica {
            id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
            display_order: Some(1),
            has_thumbnail: true,
            original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
            mime_type: "image/png".to_string(),
            created_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
        });
    }

    #[tokio::test]
    async fn create_replica_fails() {
        let mock_media_repository = MockMediaRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let mut mock_replicas_repository = MockReplicasRepository::new();
        mock_replicas_repository
            .expect_create()
            .times(1)
            .withf(|medium_id, thumbnail, original_url, mime_type| {
                (medium_id, thumbnail, original_url, mime_type) == (
                    &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    &Some(vec![0x01, 0x02, 0x03, 0x04]),
                    "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png",
                    "image/png",
                )
            })
            .returning(|_, _, _, _| Err(anyhow!("error creating a replica")));

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.create_replica(
            MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
            Some(vec![0x01, 0x02, 0x03, 0x04]),
            "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png",
            "image/png",
        ).await;

        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn create_source_succeeds() {
        let mock_media_repository = MockMediaRepository::new();
        let mock_replicas_repository = MockReplicasRepository::new();

        let mut mock_sources_repository = MockSourcesRepository::new();
        mock_sources_repository
            .expect_create()
            .times(1)
            .withf(|external_service_id, external_metadata| {
                (external_service_id, external_metadata) == (
                    &ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    &ExternalMetadata::Twitter { id: 727620202049900544 },
                )
            })
            .returning(|_, _| {
                Ok(Source {
                    id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                        slug: "twitter".to_string(),
                        name: "Twitter".to_string(),
                    },
                    external_metadata: ExternalMetadata::Twitter { id: 727620202049900544 },
                    created_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 1)).unwrap(),
                })
            });

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.create_source(
            ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            ExternalMetadata::Twitter { id: 727620202049900544 },
        ).await.unwrap();

        assert_eq!(actual, Source {
            id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                slug: "twitter".to_string(),
                name: "Twitter".to_string(),
            },
            external_metadata: ExternalMetadata::Twitter { id: 727620202049900544 },
            created_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 0)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 1)).unwrap(),
        });
    }

    #[tokio::test]
    async fn create_source_fails() {
        let mock_media_repository = MockMediaRepository::new();
        let mock_replicas_repository = MockReplicasRepository::new();

        let mut mock_sources_repository = MockSourcesRepository::new();
        mock_sources_repository
            .expect_create()
            .times(1)
            .withf(|external_service_id, external_metadata| {
                (external_service_id, external_metadata) == (
                    &ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    &ExternalMetadata::Twitter { id: 727620202049900544 },
                )
            })
            .returning(|_, _| Err(anyhow!("error creating a source")));

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.create_source(
            ExternalServiceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            ExternalMetadata::Twitter { id: 727620202049900544 },
        ).await;

        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn get_media_succeeds() {
        let mut mock_media_repository = MockMediaRepository::new();
        mock_media_repository
            .expect_fetch_all()
            .times(1)
            .withf(|tag_depth, replicas, sources, since, until, order, limit| {
                (tag_depth, replicas, sources, since, until, order, limit) == (
                    &Some(TagDepth::new(1, 1)),
                    &true,
                    &true,
                    &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
                    &None,
                    &OrderDirection::Ascending,
                    &10,
                )
            })
            .returning(|_, _, _, _, _, _, _| {
                Ok(vec![
                    Medium {
                        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                    },
                    Medium {
                        id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                    },
                    Medium {
                        id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                    },
                ])
            });

        let mock_replicas_repository = MockReplicasRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.get_media(
            Some(TagDepth::new(1, 1)),
            true,
            true,
            Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
            None,
            OrderDirection::Ascending,
            10
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
            },
            Medium {
                id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
            },
            Medium {
                id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
            },
        ]);
    }

    #[tokio::test]
    async fn get_media_fails() {
        let mut mock_media_repository = MockMediaRepository::new();
        mock_media_repository
            .expect_fetch_all()
            .times(1)
            .withf(|tag_depth, replicas, sources, since, until, order, limit| {
                (tag_depth, replicas, sources, since, until, order, limit) == (
                    &Some(TagDepth::new(1, 1)),
                    &true,
                    &true,
                    &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
                    &None,
                    &OrderDirection::Ascending,
                    &10,
                )
            })
            .returning(|_, _, _, _, _, _, _| Err(anyhow!("error fetching media")));

        let mock_replicas_repository = MockReplicasRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.get_media(
            Some(TagDepth::new(1, 1)),
            true,
            true,
            Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
            None,
            OrderDirection::Ascending,
            10
        ).await;

        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn get_media_by_ids_succeeds() {
        let mut mock_media_repository = MockMediaRepository::new();
        mock_media_repository
            .expect_fetch_by_ids()
            .times(1)
            .withf(|ids, tag_depth, replicas, sources| {
                (ids, tag_depth, replicas, sources) == (
                    &vec![
                        MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                        MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    ],
                    &Some(TagDepth::new(1, 1)),
                    &true,
                    &true,
                )
            })
            .returning(|_, _, _, _| {
                Ok(vec![
                    Medium {
                        id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                    },
                    Medium {
                        id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                    },
                ])
            });

        let mock_replicas_repository = MockReplicasRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.get_media_by_ids(
            vec![
                MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
            ],
            Some(TagDepth::new(1, 1)),
            true,
            true,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
            },
            Medium {
                id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
            },
        ]);
    }

    #[tokio::test]
    async fn get_media_by_ids_fails() {
        let mut mock_media_repository = MockMediaRepository::new();
        mock_media_repository
            .expect_fetch_by_ids()
            .times(1)
            .withf(|ids, tag_depth, replicas, sources| {
                (ids, tag_depth, replicas, sources) == (
                    &vec![
                        MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                        MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                    ],
                    &Some(TagDepth::new(1, 1)),
                    &true,
                    &true,
                )
            })
            .returning(|_, _, _, _| Err(anyhow!("error fetching the media")));

        let mock_replicas_repository = MockReplicasRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.get_media_by_ids(
            vec![
                MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
            ],
            Some(TagDepth::new(1, 1)),
            true,
            true,
        ).await;

        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn get_media_by_source_ids_succeeds() {
        let mut mock_media_repository = MockMediaRepository::new();
        mock_media_repository
            .expect_fetch_by_source_ids()
            .times(1)
            .withf(|source_ids, tag_depth, replicas, sources, since, until, order, limit| {
                (source_ids, tag_depth, replicas, sources, since, until, order, limit) == (
                    &vec![
                        SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                        SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    ],
                    &Some(TagDepth::new(1, 1)),
                    &true,
                    &true,
                    &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
                    &None,
                    &OrderDirection::Ascending,
                    &10,
                )
            })
            .returning(|_, _, _, _, _, _, _, _| {
                Ok(vec![
                    Medium {
                        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                    },
                    Medium {
                        id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
                    },
                ])
            });

        let mock_replicas_repository = MockReplicasRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.get_media_by_source_ids(
            vec![
                SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            ],
            Some(TagDepth::new(1, 1)),
            true,
            true,
            Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
            None,
            OrderDirection::Ascending,
            10
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
            },
            Medium {
                id: MediumId::from(uuid!("88888888-8888-8888-8888-888888888888")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 57)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 1)).unwrap(),
            },
        ]);
    }

    #[tokio::test]
    async fn get_media_by_source_ids_fails() {
        let mut mock_media_repository = MockMediaRepository::new();
        mock_media_repository
            .expect_fetch_by_source_ids()
            .times(1)
            .withf(|source_ids, tag_depth, replicas, sources, since, until, order, limit| {
                (source_ids, tag_depth, replicas, sources, since, until, order, limit) == (
                    &vec![
                        SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                        SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    ],
                    &Some(TagDepth::new(1, 1)),
                    &true,
                    &true,
                    &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
                    &None,
                    &OrderDirection::Ascending,
                    &10,
                )
            })
            .returning(|_, _, _, _, _, _, _, _| Err(anyhow!("error fetching the media")));

        let mock_replicas_repository = MockReplicasRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.get_media_by_source_ids(
            vec![
                SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            ],
            Some(TagDepth::new(1, 1)),
            true,
            true,
            Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
            None,
            OrderDirection::Ascending,
            10
        ).await;

        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn get_media_by_tag_ids_succeeds() {
        let mut mock_media_repository = MockMediaRepository::new();
        mock_media_repository
            .expect_fetch_by_tag_ids()
            .times(1)
            .withf(|tag_tag_type_ids, tag_depth, replicas, sources, since, until, order, limit| {
                (tag_tag_type_ids, tag_depth, replicas, sources, since, until, order, limit) == (
                    &vec![
                        (
                            TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                            TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                        ),
                        (
                            TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                            TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                        ),
                    ],
                    &Some(TagDepth::new(1, 1)),
                    &true,
                    &true,
                    &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
                    &None,
                    &OrderDirection::Ascending,
                    &10,
                )
            })
            .returning(|_, _, _, _, _, _, _, _| {
                Ok(vec![
                    Medium {
                        id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                    },
                    Medium {
                        id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                        sources: Vec::new(),
                        tags: BTreeMap::new(),
                        replicas: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
                    },
                ])
            });

        let mock_replicas_repository = MockReplicasRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.get_media_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
                (
                    TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
            ],
            Some(TagDepth::new(1, 1)),
            true,
            true,
            Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
            None,
            OrderDirection::Ascending,
            10
        ).await.unwrap();

        assert_eq!(actual, vec![
            Medium {
                id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
            },
            Medium {
                id: MediumId::from(uuid!("99999999-9999-9999-9999-999999999999")),
                sources: Vec::new(),
                tags: BTreeMap::new(),
                replicas: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 58)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 2)).unwrap(),
            },
        ]);
    }

    #[tokio::test]
    async fn get_media_by_tag_ids_fails() {
        let mut mock_media_repository = MockMediaRepository::new();
        mock_media_repository
            .expect_fetch_by_tag_ids()
            .times(1)
            .withf(|tag_tag_type_ids, tag_depth, replicas, sources, since, until, order, limit| {
                (tag_tag_type_ids, tag_depth, replicas, sources, since, until, order, limit) == (
                    &vec![
                        (
                            TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                            TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                        ),
                        (
                            TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                            TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                        ),
                    ],
                    &Some(TagDepth::new(1, 1)),
                    &true,
                    &true,
                    &Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
                    &None,
                    &OrderDirection::Ascending,
                    &10,
                )
            })
            .returning(|_, _, _, _, _, _, _, _| Err(anyhow!("error fetching the media")));

        let mock_replicas_repository = MockReplicasRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.get_media_by_tag_ids(
            vec![
                (
                    TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
                (
                    TagId::from(uuid!("55555555-5555-5555-5555-555555555555")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
            ],
            Some(TagDepth::new(1, 1)),
            true,
            true,
            Some((NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(), MediumId::from(uuid!("11111111-1111-1111-1111-111111111111")))),
            None,
            OrderDirection::Ascending,
            10
        ).await;

        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn get_replicas_by_ids_succeeds() {
        let mock_media_repository = MockMediaRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let mut mock_replicas_repository = MockReplicasRepository::new();
        mock_replicas_repository
            .expect_fetch_by_ids()
            .times(1)
            .withf(|ids: &Vec<_>| {
                ids == &vec![
                    ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                    ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                ]
            })
            .returning(|_| {
                Ok(vec![
                    Replica {
                        id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                        display_order: Some(1),
                        has_thumbnail: true,
                        original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                    },
                    Replica {
                        id: ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        display_order: Some(2),
                        has_thumbnail: true,
                        original_url: "file:///var/lib/hoarder/99999999-9999-9999-9999-999999999999.png".to_string(),
                        mime_type: "image/png".to_string(),
                        created_at: NaiveDate::from_ymd_opt(2022, 6, 3).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 6, 3).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
                    },
                ])
            });

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.get_replicas_by_ids(vec![
            ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
            ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        ]).await.unwrap();

        assert_eq!(actual, vec![
            Replica {
                id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                display_order: Some(1),
                has_thumbnail: true,
                original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
                mime_type: "image/png".to_string(),
                created_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
            },
            Replica {
                id: ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                display_order: Some(2),
                has_thumbnail: true,
                original_url: "file:///var/lib/hoarder/99999999-9999-9999-9999-999999999999.png".to_string(),
                mime_type: "image/png".to_string(),
                created_at: NaiveDate::from_ymd_opt(2022, 6, 3).and_then(|d| d.and_hms_opt(0, 2, 0)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 6, 3).and_then(|d| d.and_hms_opt(0, 3, 0)).unwrap(),
            },
        ]);
    }

    #[tokio::test]
    async fn get_replicas_by_ids_fails() {
        let mock_media_repository = MockMediaRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let mut mock_replicas_repository = MockReplicasRepository::new();
        mock_replicas_repository
            .expect_fetch_by_ids()
            .times(1)
            .withf(|ids: &Vec<_>| {
                ids == &vec![
                    ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                    ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                ]
            })
            .returning(|_| Err(anyhow!("error fetching the replicas")));

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.get_replicas_by_ids(vec![
            ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
            ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
        ]).await;

        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn get_replica_by_original_url_succeeds() {
        let mock_media_repository = MockMediaRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let mut mock_replicas_repository = MockReplicasRepository::new();
        mock_replicas_repository
            .expect_fetch_by_original_url()
            .times(1)
            .withf(|original_url| original_url == "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png")
            .returning(|_| {
                Ok(Replica {
                    id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                    display_order: Some(1),
                    has_thumbnail: true,
                    original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                })
            });

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.get_replica_by_original_url("file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png").await.unwrap();

        assert_eq!(actual, Replica {
            id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
            display_order: Some(1),
            has_thumbnail: true,
            original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
            mime_type: "image/png".to_string(),
            created_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
        });
    }

    #[tokio::test]
    async fn get_replica_by_original_url_fails() {
        let mock_media_repository = MockMediaRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let mut mock_replicas_repository = MockReplicasRepository::new();
        mock_replicas_repository
            .expect_fetch_by_original_url()
            .times(1)
            .withf(|original_url| original_url == "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png")
            .returning(|_| Err(anyhow!("error fetching the replica")));

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.get_replica_by_original_url("file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png").await;

        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn get_source_by_external_metadata_succeeds() {
        let mock_media_repository = MockMediaRepository::new();
        let mock_replicas_repository = MockReplicasRepository::new();

        let mut mock_sources_repository = MockSourcesRepository::new();
        mock_sources_repository
            .expect_fetch_by_external_metadata()
            .times(1)
            .withf(|external_service_id, external_metadata| {
                 (external_service_id, external_metadata) == (
                     &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                     &ExternalMetadata::Pixiv { id: 56736941 },
                 )
            })
            .returning(|_, _| {
                Ok(Source {
                    id: SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                        slug: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
                    created_at: NaiveDate::from_ymd_opt(2016, 5, 6).and_then(|d| d.and_hms_opt(5, 14, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2016, 5, 6).and_then(|d| d.and_hms_opt(5, 14, 1)).unwrap(),
                })
            });

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.get_source_by_external_metadata(
             ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
             ExternalMetadata::Pixiv { id: 56736941 },
        ).await.unwrap();

        assert_eq!(actual, Source {
            id: SourceId::from(uuid!("22222222-2222-2222-2222-222222222222")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                slug: "pixiv".to_string(),
                name: "pixiv".to_string(),
            },
            external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
            created_at: NaiveDate::from_ymd_opt(2016, 5, 6).and_then(|d| d.and_hms_opt(5, 14, 0)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2016, 5, 6).and_then(|d| d.and_hms_opt(5, 14, 1)).unwrap(),
        });
    }

    #[tokio::test]
    async fn get_sources_by_external_metadata_fails() {
        let mock_media_repository = MockMediaRepository::new();
        let mock_replicas_repository = MockReplicasRepository::new();

        let mut mock_sources_repository = MockSourcesRepository::new();
        mock_sources_repository
            .expect_fetch_by_external_metadata()
            .times(1)
            .withf(|external_service_id, external_metadata| {
                 (external_service_id, external_metadata) == (
                     &ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                     &ExternalMetadata::Pixiv { id: 56736941 },
                 )
            })
            .returning(|_, _| Err(anyhow!("error fetching the sources")));

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.get_source_by_external_metadata(
             ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
             ExternalMetadata::Pixiv { id: 56736941 },
        ).await;

        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn get_thumbnail_by_id_succeeds() {
        let mock_media_repository = MockMediaRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let mut mock_replicas_repository = MockReplicasRepository::new();
        mock_replicas_repository
            .expect_fetch_thumbnail_by_id()
            .times(1)
            .withf(|id| id == &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")))
            .returning(|_| {
                Ok(ReplicaThumbnail {
                    id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                    display_order: Some(1),
                    thumbnail: Some(vec![0x01, 0x02, 0x03, 0x04]),
                    original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
                    mime_type: "image/png".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                })
            });

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.get_thumbnail_by_id(ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666"))).await.unwrap();

        assert_eq!(actual, ReplicaThumbnail {
            id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
            display_order: Some(1),
            thumbnail: Some(vec![0x01, 0x02, 0x03, 0x04]),
            original_url: "file:///var/lib/hoarder/77777777-7777-7777-7777-777777777777.png".to_string(),
            mime_type: "image/png".to_string(),
            created_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
        });
    }

    #[tokio::test]
    async fn get_thumbnail_by_id_fails() {
        let mock_media_repository = MockMediaRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let mut mock_replicas_repository = MockReplicasRepository::new();
        mock_replicas_repository
            .expect_fetch_thumbnail_by_id()
            .times(1)
            .withf(|id| id == &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")))
            .returning(|_| Err(anyhow!("error fetching the replica")));

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.get_thumbnail_by_id(ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666"))).await;

        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn update_medium_by_id_succeeds() {
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
                (
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
                ) == (
                    &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    &[
                        SourceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    ],
                    &[
                        SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    ],
                    &[
                        (
                            TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                            TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                        ),
                    ],
                    &[
                        (
                            TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                            TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                        ),
                    ],
                    &[
                        ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                    ],
                    &Some(NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap()),
                    &Some(TagDepth::new(1, 1)),
                    &true,
                    &true,
                )
            })
            .returning(|_, _, _, _, _, _, _, _, _, _| {
                Ok(Medium {
                    id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    sources: Vec::new(),
                    tags: BTreeMap::new(),
                    replicas: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
                })
            });

        let mock_replicas_repository = MockReplicasRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.update_medium_by_id(
            MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
            [
                SourceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            ],
            [
                SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            ],
            [
                (
                    TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
            ],
            [
                (
                    TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
            ],
            [
                ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
            ],
            Some(NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap()),
            Some(TagDepth::new(1, 1)),
            true,
            true,
        ).await.unwrap();

        assert_eq!(actual, Medium {
            id: MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
            sources: Vec::new(),
            tags: BTreeMap::new(),
            replicas: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(0, 5, 0)).unwrap(),
        });
    }

    #[tokio::test]
    async fn update_medium_by_id_fails() {
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
                (
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
                ) == (
                    &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                    &[
                        SourceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    ],
                    &[
                        SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    ],
                    &[
                        (
                            TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                            TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                        ),
                    ],
                    &[
                        (
                            TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                            TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                        ),
                    ],
                    &[
                        ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                        ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                    ],
                    &Some(NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap()),
                    &Some(TagDepth::new(1, 1)),
                    &true,
                    &true,
                )
            })
            .returning(|_, _, _, _, _, _, _, _, _, _| Err(anyhow!("error updating the medium")));

        let mock_replicas_repository = MockReplicasRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.update_medium_by_id(
            MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")),
            [
                SourceId::from(uuid!("33333333-3333-3333-3333-333333333333")),
            ],
            [
                SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            ],
            [
                (
                    TagId::from(uuid!("22222222-2222-2222-2222-222222222222")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
            ],
            [
                (
                    TagId::from(uuid!("33333333-3333-3333-3333-333333333333")),
                    TagTypeId::from(uuid!("44444444-4444-4444-4444-444444444444")),
                ),
            ],
            [
                ReplicaId::from(uuid!("77777777-7777-7777-7777-777777777777")),
                ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
            ],
            Some(NaiveDate::from_ymd_opt(2022, 6, 1).and_then(|d| d.and_hms_opt(12, 34, 56)).unwrap()),
            Some(TagDepth::new(1, 1)),
            true,
            true,
        ).await;

        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn update_replica_by_id_succeeds() {
        let mock_media_repository = MockMediaRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();
        let mut mock_replicas_repository = MockReplicasRepository::new();
        mock_replicas_repository
            .expect_update_by_id()
            .times(1)
            .withf(|id, thumbnail, original_url, mime_type| {
                (id, thumbnail, original_url, mime_type) == (
                    &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                    &Some(vec![0x01, 0x02, 0x03, 0x04]),
                    &Some("file:///var/lib/hoarder/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg"),
                    &Some("image/jpeg"),
                )
            })
            .returning(|_, _, _, _| {
                Ok(Replica {
                    id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                    display_order: Some(1),
                    has_thumbnail: true,
                    original_url: "file:///var/lib/hoarder/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
                    mime_type: "image/jpeg".to_string(),
                    created_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
                })
            });

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.update_replica_by_id(
            ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
            Some(vec![0x01, 0x02, 0x03, 0x04]),
            Some("file:///var/lib/hoarder/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg"),
            Some("image/jpeg"),
        ).await.unwrap();

        assert_eq!(actual, Replica {
            id: ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
            display_order: Some(1),
            has_thumbnail: true,
            original_url: "file:///var/lib/hoarder/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            created_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 6, 2).and_then(|d| d.and_hms_opt(0, 1, 0)).unwrap(),
        });
    }

    #[tokio::test]
    async fn update_replica_by_id_fails() {
        let mock_media_repository = MockMediaRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();
        let mut mock_replicas_repository = MockReplicasRepository::new();
        mock_replicas_repository
            .expect_update_by_id()
            .times(1)
            .withf(|id, thumbnail, original_url, mime_type| {
                (id, thumbnail, original_url, mime_type) == (
                    &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
                    &Some(vec![0x01, 0x02, 0x03, 0x04]),
                    &Some("file:///var/lib/hoarder/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg"),
                    &Some("image/jpeg"),
                )
            })
            .returning(|_, _, _, _| Err(anyhow!("error updating the replica")));

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.update_replica_by_id(
            ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")),
            Some(vec![0x01, 0x02, 0x03, 0x04]),
            Some("file:///var/lib/hoarder/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.jpg"),
            Some("image/jpeg"),
        ).await;

        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn update_source_by_id_succeeds() {
        let mock_media_repository = MockMediaRepository::new();
        let mock_replicas_repository = MockReplicasRepository::new();

        let mut mock_sources_repository = MockSourcesRepository::new();
        mock_sources_repository
            .expect_update_by_id()
            .times(1)
            .withf(|id, external_service_id, external_metadata| {
                (id, external_service_id, external_metadata) == (
                    &SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    &Some(ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111"))),
                    &Some(ExternalMetadata::Pixiv { id: 56736941 }),
                )
            })
            .returning(|_, _, _| {
                Ok(Source {
                    id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    external_service: ExternalService {
                        id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                        slug: "pixiv".to_string(),
                        name: "pixiv".to_string(),
                    },
                    external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
                    created_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 0)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 1)).unwrap(),
                })
            });

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.update_source_by_id(
            SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            Some(ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111"))),
            Some(ExternalMetadata::Pixiv { id: 56736941 }),
        ).await.unwrap();

        assert_eq!(actual, Source {
            id: SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            external_service: ExternalService {
                id: ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                slug: "pixiv".to_string(),
                name: "pixiv".to_string(),
            },
            external_metadata: ExternalMetadata::Pixiv { id: 56736941 },
            created_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 0)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2016, 5, 4).and_then(|d| d.and_hms_opt(7, 5, 1)).unwrap(),
        });
    }

    #[tokio::test]
    async fn update_source_by_id_fails() {
        let mock_media_repository = MockMediaRepository::new();
        let mock_replicas_repository = MockReplicasRepository::new();

        let mut mock_sources_repository = MockSourcesRepository::new();
        mock_sources_repository
            .expect_update_by_id()
            .times(1)
            .withf(|id, external_service_id, external_metadata| {
                (id, external_service_id, external_metadata) == (
                    &SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
                    &Some(ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111"))),
                    &Some(ExternalMetadata::Pixiv { id: 56736941 }),
                )
            })
            .returning(|_, _, _| Err(anyhow!("error updating the source")));

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.update_source_by_id(
            SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            Some(ExternalServiceId::from(uuid!("11111111-1111-1111-1111-111111111111"))),
            Some(ExternalMetadata::Pixiv { id: 56736941 }),
        ).await;

        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn delete_medium_by_id_succeeds() {
        let mut mock_media_repository = MockMediaRepository::new();
        mock_media_repository
            .expect_delete_by_id()
            .times(1)
            .withf(|id| id == &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")))
            .returning(|_| Ok(DeleteResult::Deleted(1)));

        let mock_replicas_repository = MockReplicasRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.delete_medium_by_id(MediumId::from(uuid!("77777777-7777-7777-7777-777777777777"))).await.unwrap();

        assert_eq!(actual, DeleteResult::Deleted(1));
    }

    #[tokio::test]
    async fn delete_medium_by_id_fails() {
        let mut mock_media_repository = MockMediaRepository::new();
        mock_media_repository
            .expect_delete_by_id()
            .times(1)
            .withf(|id| id == &MediumId::from(uuid!("77777777-7777-7777-7777-777777777777")))
            .returning(|_| Err(anyhow!("error deleting the medium")));

        let mock_replicas_repository = MockReplicasRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.delete_medium_by_id(MediumId::from(uuid!("77777777-7777-7777-7777-777777777777"))).await;

        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn delete_replica_by_id_succeeds() {
        let mock_media_repository = MockMediaRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let mut mock_replicas_repository = MockReplicasRepository::new();
        mock_replicas_repository
            .expect_delete_by_id()
            .times(1)
            .withf(|id| id == &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")))
            .returning(|_| Ok(DeleteResult::Deleted(1)));

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.delete_replica_by_id(ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666"))).await.unwrap();

        assert_eq!(actual, DeleteResult::Deleted(1));
    }

    #[tokio::test]
    async fn delete_replica_by_id_fails() {
        let mock_media_repository = MockMediaRepository::new();
        let mock_sources_repository = MockSourcesRepository::new();

        let mut mock_replicas_repository = MockReplicasRepository::new();
        mock_replicas_repository
            .expect_delete_by_id()
            .times(1)
            .withf(|id| id == &ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666")))
            .returning(|_| Err(anyhow!("error deleting the replica")));

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.delete_replica_by_id(ReplicaId::from(uuid!("66666666-6666-6666-6666-666666666666"))).await;

        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn delete_source_by_id_succeeds() {
        let mock_media_repository = MockMediaRepository::new();
        let mock_replicas_repository = MockReplicasRepository::new();

        let mut mock_sources_repository = MockSourcesRepository::new();
        mock_sources_repository
            .expect_delete_by_id()
            .times(1)
            .withf(|id| id == &SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")))
            .returning(|_| Ok(DeleteResult::Deleted(1)));

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.delete_source_by_id(SourceId::from(uuid!("11111111-1111-1111-1111-111111111111"))).await.unwrap();

        assert_eq!(actual, DeleteResult::Deleted(1));
    }

    #[tokio::test]
    async fn delete_source_by_id_fails() {
        let mock_media_repository = MockMediaRepository::new();
        let mock_replicas_repository = MockReplicasRepository::new();

        let mut mock_sources_repository = MockSourcesRepository::new();
        mock_sources_repository
            .expect_delete_by_id()
            .times(1)
            .withf(|id| id == &SourceId::from(uuid!("11111111-1111-1111-1111-111111111111")))
            .returning(|_| Err(anyhow!("error deleting the source")));

        let service = MediaService::new(mock_media_repository, mock_replicas_repository, mock_sources_repository);
        let actual = service.delete_source_by_id(SourceId::from(uuid!("11111111-1111-1111-1111-111111111111"))).await;

        assert!(actual.is_err());
    }
}
