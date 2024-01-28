use std::future::Future;

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use derive_more::Constructor;
use tokio::try_join;

use crate::{
    entity::{
        external_services::{ExternalMetadata, ExternalServiceId},
        media::{Medium, MediumId},
        replicas::{OriginalImage, Replica, ReplicaId, ThumbnailId, ThumbnailImage},
        sources::{Source, SourceId},
        tag_types::TagTypeId,
        tags::{TagDepth, TagId},
    },
    parser, processor,
    repository::{media, replicas, sources, DeleteResult, Direction, Order},
};

#[cfg_attr(feature = "test-mock", mockall::automock)]
pub trait MediaServiceInterface: Send + Sync + 'static {
    /// Creates a medium.
    fn create_medium<T, U>(&self, source_ids: T, created_at: Option<DateTime<Utc>>, tag_tag_type_ids: U, tag_depth: Option<TagDepth>, sources: bool) -> impl Future<Output = anyhow::Result<Medium>> + Send
    where
        T: IntoIterator<Item = SourceId> + Send + Sync + 'static,
        U: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static;

    /// Creates a replica.
    fn create_replica(&self, medium_id: MediumId, original_url: &str) -> impl Future<Output = anyhow::Result<Replica>> + Send;

    /// Creates a source.
    fn create_source(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> impl Future<Output = anyhow::Result<Source>> + Send;

    /// Gets media.
    fn get_media(
        &self,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
        cursor: Option<(DateTime<Utc>, MediumId)>,
        order: Order,
        direction: Direction,
        limit: u64,
    ) -> impl Future<Output = anyhow::Result<Vec<Medium>>> + Send;

    /// Gets the media by their IDs.
    fn get_media_by_ids<T>(&self, ids: T, tag_depth: Option<TagDepth>, replicas: bool, sources: bool) -> impl Future<Output = anyhow::Result<Vec<Medium>>> + Send
    where
        T: IntoIterator<Item = MediumId> + Send + Sync + 'static;

    /// Gets the media by their source IDs.
    fn get_media_by_source_ids<T>(
        &self,
        source_ids: T,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
        cursor: Option<(DateTime<Utc>, MediumId)>,
        order: Order,
        direction: Direction,
        limit: u64,
    ) -> impl Future<Output = anyhow::Result<Vec<Medium>>> + Send
    where
        T: IntoIterator<Item = SourceId> + Send + Sync + 'static;

    /// Gets the media by their tag IDs.
    fn get_media_by_tag_ids<T>(
        &self,
        tag_tag_type_ids: T,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
        cursor: Option<(DateTime<Utc>, MediumId)>,
        order: Order,
        direction: Direction,
        limit: u64,
    ) -> impl Future<Output = anyhow::Result<Vec<Medium>>> + Send
    where
        T: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static;

    /// Gets the replicas by their IDs.
    fn get_replicas_by_ids<T>(&self, ids: T) -> impl Future<Output = anyhow::Result<Vec<Replica>>> + Send
    where
        T: IntoIterator<Item = ReplicaId> + Send + Sync + 'static;

    /// Gets the replica by original URL.
    fn get_replica_by_original_url(&self, original_url: &str) -> impl Future<Output = anyhow::Result<Replica>> + Send;

    /// Gets the source by its external metadata.
    fn get_source_by_external_metadata(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> impl Future<Output = anyhow::Result<Option<Source>>> + Send;

    /// Gets the by ID.
    fn get_thumbnail_by_id(&self, id: ThumbnailId) -> impl Future<Output = anyhow::Result<Vec<u8>>> + Send;

    /// Updates the medium by ID.
    fn update_medium_by_id<T, U, V, W, X>(
        &self,
        id: MediumId,
        add_source_ids: T,
        remove_source_ids: U,
        add_tag_tag_type_ids: V,
        remove_tag_tag_type_ids: W,
        replica_orders: X,
        created_at: Option<DateTime<Utc>>,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
    ) -> impl Future<Output = anyhow::Result<Medium>> + Send
    where
        T: IntoIterator<Item = SourceId> + Send + Sync + 'static,
        U: IntoIterator<Item = SourceId> + Send + Sync + 'static,
        V: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
        W: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
        X: IntoIterator<Item = ReplicaId> + Send + Sync + 'static;

    /// Updates the replica by ID.
    fn update_replica_by_id<'a>(&self, id: ReplicaId, original_url: Option<&'a str>) -> impl Future<Output = anyhow::Result<Replica>> + Send;

    /// Updates the source by ID.
    fn update_source_by_id(&self, id: SourceId, external_service_id: Option<ExternalServiceId>, external_metadata: Option<ExternalMetadata>) -> impl Future<Output = anyhow::Result<Source>> + Send;

    /// Deletes the medium by ID.
    fn delete_medium_by_id(&self, id: MediumId) -> impl Future<Output = anyhow::Result<DeleteResult>> + Send;

    /// Deletes the replica by ID.
    fn delete_replica_by_id(&self, id: ReplicaId) -> impl Future<Output = anyhow::Result<DeleteResult>> + Send;

    /// Deletes the source by ID.
    fn delete_source_by_id(&self, id: SourceId) -> impl Future<Output = anyhow::Result<DeleteResult>> + Send;
}

#[derive(Clone, Constructor)]
pub struct MediaService<MediaRepository, ReplicasRepository, SourcesRepository, MediumImageParser, MediumImageProcessor> {
    media_repository: MediaRepository,
    replicas_repository: ReplicasRepository,
    sources_repository: SourcesRepository,
    medium_image_parser: MediumImageParser,
    medium_image_processor: MediumImageProcessor,
}

fn extract_file_path(url: &str) -> anyhow::Result<&str> {
    match url.split_once("://") {
        Some(("file", path)) => Ok(path),
        Some((scheme, _)) => Err(anyhow!("unsupported scheme: {}", scheme)),
        None => Err(anyhow!("unsupported url: {}", url)),
    }
}

impl<MediaRepository, ReplicasRepository, SourcesRepository, MediumImageParser, MediumImageProcessor> MediaService<MediaRepository, ReplicasRepository, SourcesRepository, MediumImageParser, MediumImageProcessor>
where
    MediumImageParser: parser::media::MediumImageParser,
{
    async fn fetch_original_image(&self, url: &str) -> anyhow::Result<OriginalImage> {
        let path = extract_file_path(url)?;
        match self.medium_image_parser.get_metadata(path).await {
            Ok(metadata) => Ok(OriginalImage::new(metadata.mime_type(), metadata.size())),
            Err(e) => {
                log::error!("failed to get the original image size\nError: {e:?}");
                Err(e)
            },
        }
    }
}

impl<MediaRepository, ReplicasRepository, SourcesRepository, MediumImageParser, MediumImageProcessor> MediaService<MediaRepository, ReplicasRepository, SourcesRepository, MediumImageParser, MediumImageProcessor>
where
    MediumImageProcessor: processor::media::MediumImageProcessor,
{
    async fn generate_thumbnail_image(&self, url: &str) -> anyhow::Result<ThumbnailImage> {
        let path = extract_file_path(url)?;
        match self.medium_image_processor.generate_thumbnail(path).await {
            Ok(thumbnail_image) => Ok(thumbnail_image),
            Err(e) => {
                log::error!("failed to generate a thumbnail image\nError: {e:?}");
                Err(e)
            },
        }
    }
}

impl<MediaRepository, ReplicasRepository, SourcesRepository, MediumImageParser, MediumImageProcessor> MediaServiceInterface for MediaService<MediaRepository, ReplicasRepository, SourcesRepository, MediumImageParser, MediumImageProcessor>
where
    MediaRepository: media::MediaRepository,
    ReplicasRepository: replicas::ReplicasRepository,
    SourcesRepository: sources::SourcesRepository,
    MediumImageParser: parser::media::MediumImageParser,
    MediumImageProcessor: processor::media::MediumImageProcessor,
{
    async fn create_medium<T, U>(&self, source_ids: T, created_at: Option<DateTime<Utc>>, tag_tag_type_ids: U, tag_depth: Option<TagDepth>, sources: bool) -> anyhow::Result<Medium>
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

    async fn create_replica(&self, medium_id: MediumId, original_url: &str) -> anyhow::Result<Replica> {
        let (thumbnail_image, original_image) = try_join!(
            self.generate_thumbnail_image(original_url),
            self.fetch_original_image(original_url),
        )?;
        match self.replicas_repository.create(medium_id, Some(thumbnail_image), original_url, original_image).await {
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
        cursor: Option<(DateTime<Utc>, MediumId)>,
        order: Order,
        direction: Direction,
        limit: u64,
    ) -> anyhow::Result<Vec<Medium>> {
        match self.media_repository.fetch_all(tag_depth, replicas, sources, cursor, order, direction, limit).await {
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
        cursor: Option<(DateTime<Utc>, MediumId)>,
        order: Order,
        direction: Direction,
        limit: u64,
    ) -> anyhow::Result<Vec<Medium>>
    where
        T: IntoIterator<Item = SourceId> + Send + Sync + 'static,
    {
        match self.media_repository.fetch_by_source_ids(source_ids, tag_depth, replicas, sources, cursor, order, direction, limit).await {
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
        cursor: Option<(DateTime<Utc>, MediumId)>,
        order: Order,
        direction: Direction,
        limit: u64,
    ) -> anyhow::Result<Vec<Medium>>
    where
        T: IntoIterator<Item = (TagId, TagTypeId)> + Send + Sync + 'static,
    {
        match self.media_repository.fetch_by_tag_ids(tag_tag_type_ids, tag_depth, replicas, sources, cursor, order, direction, limit).await {
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

    async fn get_source_by_external_metadata(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> anyhow::Result<Option<Source>> {
        match self.sources_repository.fetch_by_external_metadata(external_service_id, external_metadata).await {
            Ok(source) => Ok(source),
            Err(e) => {
                log::error!("failed to get the source\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_thumbnail_by_id(&self, id: ThumbnailId) -> anyhow::Result<Vec<u8>> {
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
        created_at: Option<DateTime<Utc>>,
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

    async fn update_replica_by_id<'a>(&self, id: ReplicaId, original_url: Option<&'a str>) -> anyhow::Result<Replica> {
        let (thumbnail_image, original_image) = match original_url {
            Some(original_url) => {
                let (thumbnail_image, original_image) = try_join!(
                    self.generate_thumbnail_image(original_url),
                    self.fetch_original_image(original_url),
                )?;
                (Some(thumbnail_image), Some(original_image))
            },
            None => (None, None),
        };
        match self.replicas_repository.update_by_id(id, thumbnail_image, original_url, original_image).await {
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
