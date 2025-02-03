use std::{future::Future, io::{BufReader, Read, Seek, SeekFrom}};

use chrono::{DateTime, Utc};
use derive_more::Constructor;
use futures::Stream;
use tokio::task::{self, JoinHandle};
use tokio_util::task::TaskTracker;

use crate::{
    entity::{
        external_services::{ExternalMetadata, ExternalServiceId},
        media::{Medium, MediumId},
        objects::{Entry, EntryKind, EntryUrl, EntryUrlPath},
        replicas::{OriginalImage, Replica, ReplicaId, ReplicaStatus, ThumbnailId, ThumbnailImage},
        sources::{Source, SourceId},
        tag_types::TagTypeId,
        tags::{TagDepth, TagId},
    },
    error::{Error, ErrorKind, Result},
    iter::CloneableIterator,
    processor,
    repository::{media, objects, replicas, sources, DeleteResult, Direction, Order},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MediumSource<R> {
    Url(EntryUrl),
    Content(EntryUrlPath, R, MediumOverwriteBehavior),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediumOverwriteBehavior {
    Overwrite,
    Fail,
}

impl From<MediumOverwriteBehavior> for objects::ObjectOverwriteBehavior {
    fn from(value: MediumOverwriteBehavior) -> Self {
        match value {
            MediumOverwriteBehavior::Overwrite => objects::ObjectOverwriteBehavior::Overwrite,
            MediumOverwriteBehavior::Fail => objects::ObjectOverwriteBehavior::Fail,
        }
    }
}

pub trait MediaServiceInterface: Send + Sync + 'static {
    /// Creates a medium.
    fn create_medium<T, U>(&self, source_ids: T, created_at: Option<DateTime<Utc>>, tag_tag_type_ids: U, tag_depth: Option<TagDepth>, sources: bool) -> impl Future<Output = Result<Medium>> + Send
    where
        T: CloneableIterator<Item = SourceId> + Send,
        U: CloneableIterator<Item = (TagId, TagTypeId)> + Send;

    /// Creates a replica.
    fn create_replica<R>(&self, medium_id: MediumId, medium_source: MediumSource<R>) -> impl Future<Output = Result<(Replica, JoinHandle<()>)>> + Send
    where
        for<'a> R: Read + Seek + Send + 'a;

    /// Creates a source.
    fn create_source(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> impl Future<Output = Result<Source>> + Send;

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
    ) -> impl Future<Output = Result<Vec<Medium>>> + Send;

    /// Gets the media by their IDs.
    fn get_media_by_ids<T>(&self, ids: T, tag_depth: Option<TagDepth>, replicas: bool, sources: bool) -> impl Future<Output = Result<Vec<Medium>>> + Send
    where
        T: CloneableIterator<Item = MediumId> + Send;

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
    ) -> impl Future<Output = Result<Vec<Medium>>> + Send
    where
        T: CloneableIterator<Item = SourceId> + Send;

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
    ) -> impl Future<Output = Result<Vec<Medium>>> + Send
    where
        T: CloneableIterator<Item = (TagId, TagTypeId)> + Send;

    /// Gets the replicas by their IDs.
    fn get_replicas_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<Replica>>> + Send
    where
        T: CloneableIterator<Item = ReplicaId> + Send;

    /// Gets the replica by original URL.
    fn get_replica_by_original_url(&self, original_url: &str) -> impl Future<Output = Result<Replica>> + Send;

    /// Gets the sourecs by their IDs.
    fn get_sources_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<Source>>> + Send
    where
        T: CloneableIterator<Item = SourceId> + Send;

    /// Gets the source by its external metadata.
    fn get_source_by_external_metadata(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> impl Future<Output = Result<Option<Source>>> + Send;

    /// Gets the sources by ID field of their external metadata.
    fn get_sources_by_external_metadata_like_id(&self, id: &str) -> impl Future<Output = Result<Vec<Source>>> + Send;

    /// Gets the by ID.
    fn get_thumbnail_by_id(&self, id: ThumbnailId) -> impl Future<Output = Result<Vec<u8>>> + Send;

    /// Gets the object by its URL.
    fn get_object(&self, url: EntryUrl) -> impl Future<Output = Result<Entry>> + Send;

    /// Gets objects.
    fn get_objects(&self, prefix: EntryUrlPath, kind: Option<EntryKind>) -> impl Future<Output = Result<Vec<Entry>>> + Send;

    /// Watches the medium by ID.
    fn watch_medium_by_id(&self, id: MediumId, tag_depth: Option<TagDepth>, replicas: bool, sources: bool) -> impl Future<Output = Result<impl Stream<Item = Result<Medium>> + Send>> + Send;

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
    ) -> impl Future<Output = Result<Medium>> + Send
    where
        T: CloneableIterator<Item = SourceId> + Send,
        U: CloneableIterator<Item = SourceId> + Send,
        V: CloneableIterator<Item = (TagId, TagTypeId)> + Send,
        W: CloneableIterator<Item = (TagId, TagTypeId)> + Send,
        X: CloneableIterator<Item = ReplicaId> + Send;

    /// Updates the replica by ID.
    fn update_replica_by_id<R>(&self, id: ReplicaId, medium_source: MediumSource<R>) -> impl Future<Output = Result<(Replica, JoinHandle<()>)>> + Send
    where
        for<'a> R: Read + Seek + Send + 'a;

    /// Updates the source by ID.
    fn update_source_by_id(&self, id: SourceId, external_service_id: Option<ExternalServiceId>, external_metadata: Option<ExternalMetadata>) -> impl Future<Output = Result<Source>> + Send;

    /// Deletes the medium by ID.
    fn delete_medium_by_id(&self, id: MediumId, delete_objects: bool) -> impl Future<Output = Result<DeleteResult>> + Send;

    /// Deletes the replica by ID.
    fn delete_replica_by_id(&self, id: ReplicaId, delete_object: bool) -> impl Future<Output = Result<DeleteResult>> + Send;

    /// Deletes the source by ID.
    fn delete_source_by_id(&self, id: SourceId) -> impl Future<Output = Result<DeleteResult>> + Send;
}

#[derive(Clone, Constructor)]
pub struct MediaService<MediaRepository, ObjectsRepository, ReplicasRepository, SourcesRepository, MediumImageProcessor> {
    media_repository: MediaRepository,
    objects_repository: ObjectsRepository,
    replicas_repository: ReplicasRepository,
    sources_repository: SourcesRepository,
    medium_image_processor: MediumImageProcessor,
    tracker: TaskTracker,
}

impl<MediaRepository, ObjectsRepository, ReplicasRepository, SourcesRepository, MediumImageProcessor> MediaService<MediaRepository, ObjectsRepository, ReplicasRepository, SourcesRepository, MediumImageProcessor>
where
    ObjectsRepository: objects::ObjectsRepository,
{
    async fn get_image(&self, url: EntryUrl) -> Result<(EntryUrl, ObjectsRepository::Read)> {
        match self.objects_repository.get(url).await {
            Ok((entry, read)) => {
                if let Some(url) = entry.url {
                    Ok((url, read))
                } else {
                    Err(ErrorKind::ObjectPathInvalid)?
                }
            },
            Err(e) => {
                log::error!("failed to get an image\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn put_image(&self, url: EntryUrl, overwrite: MediumOverwriteBehavior) -> Result<(EntryUrl, objects::ObjectStatus, ObjectsRepository::Write)> {
        match self.objects_repository.put(url, overwrite.into()).await {
            Ok((entry, status, write)) => {
                if let Some(url) = entry.url {
                    Ok((url, status, write))
                } else {
                    Err(ErrorKind::ObjectPathInvalid)?
                }
            },
            Err(e) => {
                log::error!("failed to put an image\nError: {e:?}");
                Err(e)
            },
        }
    }
}

impl<MediaRepository, ObjectsRepository, ReplicasRepository, SourcesRepository, MediumImageProcessor> MediaService<MediaRepository, ObjectsRepository, ReplicasRepository, SourcesRepository, MediumImageProcessor>
where
    MediumImageProcessor: processor::media::MediumImageProcessor + Clone,
    ObjectsRepository: objects::ObjectsRepository + Clone,
{
    async fn extract_medium_source<R>(&self, medium_source: MediumSource<R>) -> Result<(EntryUrl, objects::ObjectStatus, Box<dyn FnOnce() -> Result<(OriginalImage, ThumbnailImage)> + Send>)>
    where
        for<'a> R: Read + Seek + Send + 'a,
    {
        let medium_image_processor = self.medium_image_processor.clone();
        match medium_source {
            MediumSource::Url(url) => {
                let (url, read) = self.get_image(url).await?;
                let read = BufReader::new(read);

                Ok((
                    url,
                    objects::ObjectStatus::Existing,
                    Box::new(move || {
                        let (original_image, thumbnail_image) = medium_image_processor.generate_thumbnail(read)?;
                        Ok((original_image, thumbnail_image))
                    }),
                ))
            },
            MediumSource::Content(path, content, overwrite) => {
                let url = path.to_url(ObjectsRepository::scheme());
                let (url, status, mut write) = self.put_image(url, overwrite).await?;

                let objects_repository = self.objects_repository.clone();
                Ok((
                    url,
                    status,
                    Box::new(move || {
                        let mut read = content;
                        objects_repository.copy(&mut read, &mut write)?;

                        let mut read = BufReader::new(read);
                        read.seek(SeekFrom::Start(0)).map_err(Error::other)?;

                        let (original_image, thumbnail_image) = medium_image_processor.generate_thumbnail(read)?;
                        Ok((original_image, thumbnail_image))
                    }),
                ))
            },
        }
    }
}

impl<MediaRepository, ObjectsRepository, ReplicasRepository, SourcesRepository, MediumImageProcessor> MediaService<MediaRepository, ObjectsRepository, ReplicasRepository, SourcesRepository, MediumImageProcessor>
where
    MediumImageProcessor: processor::media::MediumImageProcessor + Clone,
    ReplicasRepository: replicas::ReplicasRepository + Clone,
    ObjectsRepository: objects::ObjectsRepository + Clone,
{
    async fn create_replica_source<R>(&self, medium_source: MediumSource<R>) -> Result<(EntryUrl, objects::ObjectStatus, Box<dyn FnOnce() -> Result<(OriginalImage, ThumbnailImage)> + Send>)>
    where
        for<'a> R: Read + Seek + Send + 'a,
    {
        match self.extract_medium_source(medium_source).await {
            Ok((url, status, process)) => Ok((url, status, process)),
            Err(e) => {
                let ErrorKind::ObjectAlreadyExists { url, entry } = e.kind() else {
                    log::error!("failed to process a medium\nError: {e:?}");
                    return Err(e);
                };
                match self.replicas_repository.fetch_by_original_url(url).await {
                    Ok(replica) => {
                        let original_url = replica.original_url;
                        let entry = entry.clone();
                        Err(Error::new(ErrorKind::ReplicaOriginalUrlDuplicate { original_url, entry }, e))
                    },
                    Err(_) => Err(e),
                }
            },
        }
    }

    fn process_replica_by_id(&self, id: ReplicaId, process: Box<dyn FnOnce() -> Result<(OriginalImage, ThumbnailImage)> + Send>) -> JoinHandle<()> {
        let replicas_repository = self.replicas_repository.clone();

        self.tracker.spawn(async move {
            let (original_image, thumbnail_image, status) = match task::spawn_blocking(process).await.map_err(Error::other).and_then(|result| result) {
                Ok((original_image, thumbnail_image)) => (Some(original_image), Some(thumbnail_image), ReplicaStatus::Ready),
                Err(e) => {
                    log::error!("failed to process a medium\nError: {e:?}");
                    (None, None, ReplicaStatus::Error)
                },
            };

            if let Err(e) = replicas_repository.update_by_id(id, Some(thumbnail_image), None, Some(original_image), Some(status)).await {
                log::error!("failed to update the replica\nError: {e:?}");
            }
        })
    }
}

impl<MediaRepository, ObjectsRepository, ReplicasRepository, SourcesRepository, MediumImageProcessor> MediaServiceInterface for MediaService<MediaRepository, ObjectsRepository, ReplicasRepository, SourcesRepository, MediumImageProcessor>
where
    MediaRepository: media::MediaRepository,
    ObjectsRepository: objects::ObjectsRepository + Clone,
    ReplicasRepository: replicas::ReplicasRepository + Clone,
    SourcesRepository: sources::SourcesRepository,
    MediumImageProcessor: processor::media::MediumImageProcessor + Clone,
{
    async fn create_medium<T, U>(&self, source_ids: T, created_at: Option<DateTime<Utc>>, tag_tag_type_ids: U, tag_depth: Option<TagDepth>, sources: bool) -> Result<Medium>
    where
        T: CloneableIterator<Item = SourceId> + Send,
        U: CloneableIterator<Item = (TagId, TagTypeId)> + Send,
    {
        match self.media_repository.create(source_ids, created_at, tag_tag_type_ids, tag_depth, sources).await {
            Ok(medium) => Ok(medium),
            Err(e) => {
                log::error!("failed to create a medium\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn create_replica<R>(&self, medium_id: MediumId, medium_source: MediumSource<R>) -> Result<(Replica, JoinHandle<()>)>
    where
        for<'a> R: Read + Seek + Send + 'a,
    {
        let (url, status, process) = self.create_replica_source(medium_source).await?;
        match self.replicas_repository.create(medium_id, None, &url, None, ReplicaStatus::Processing).await {
            Ok(replica) => {
                let handle = self.process_replica_by_id(replica.id, process);
                Ok((replica, handle))
            },
            Err(e) if status.is_created() => {
                log::error!("failed to create a replica\nError: {e:?}");

                if let Err(e) = self.objects_repository.delete(url).await {
                    log::error!("failed to delete the object\nError: {e:?}");
                }
                Err(e)
            },
            Err(e) => {
                log::error!("failed to create a replica\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn create_source(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> Result<Source> {
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
    ) -> Result<Vec<Medium>> {
        match self.media_repository.fetch_all(tag_depth, replicas, sources, cursor, order, direction, limit).await {
            Ok(media) => Ok(media),
            Err(e) => {
                log::error!("failed to get the media\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_media_by_ids<T>(&self, ids: T, tag_depth: Option<TagDepth>, replicas: bool, sources: bool) -> Result<Vec<Medium>>
    where
        T: CloneableIterator<Item = MediumId> + Send,
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
    ) -> Result<Vec<Medium>>
    where
        T: CloneableIterator<Item = SourceId> + Send,
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
    ) -> Result<Vec<Medium>>
    where
        T: CloneableIterator<Item = (TagId, TagTypeId)> + Send,
    {
        match self.media_repository.fetch_by_tag_ids(tag_tag_type_ids, tag_depth, replicas, sources, cursor, order, direction, limit).await {
            Ok(media) => Ok(media),
            Err(e) => {
                log::error!("failed to get the media\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_replicas_by_ids<T>(&self, ids: T) -> Result<Vec<Replica>>
    where
        T: CloneableIterator<Item = ReplicaId> + Send,
    {
        match self.replicas_repository.fetch_by_ids(ids).await {
            Ok(replicas) => Ok(replicas),
            Err(e) => {
                log::error!("failed to get the replicas\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_replica_by_original_url(&self, original_url: &str) -> Result<Replica> {
        match self.replicas_repository.fetch_by_original_url(original_url).await {
            Ok(replica) => Ok(replica),
            Err(e) => {
                log::error!("failed to get the replica\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_sources_by_ids<T>(&self, ids: T) -> Result<Vec<Source>>
    where
        T: CloneableIterator<Item = SourceId> + Send,
    {
        match self.sources_repository.fetch_by_ids(ids).await {
            Ok(sources) => Ok(sources),
            Err(e) => {
                log::error!("failed to get the sources\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_source_by_external_metadata(&self, external_service_id: ExternalServiceId, external_metadata: ExternalMetadata) -> Result<Option<Source>> {
        match self.sources_repository.fetch_by_external_metadata(external_service_id, external_metadata).await {
            Ok(source) => Ok(source),
            Err(e) => {
                log::error!("failed to get the source\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_sources_by_external_metadata_like_id(&self, id: &str) -> Result<Vec<Source>> {
        match self.sources_repository.fetch_by_external_metadata_like_id(id).await {
            Ok(sources) => Ok(sources),
            Err(e) => {
                log::error!("failed to get the sources\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_thumbnail_by_id(&self, id: ThumbnailId) -> Result<Vec<u8>> {
        match self.replicas_repository.fetch_thumbnail_by_id(id).await {
            Ok(replica) => Ok(replica),
            Err(e) => {
                log::error!("failed to get the replica\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_object(&self, url: EntryUrl) -> Result<Entry> {
        match self.objects_repository.get(url).await {
            Ok((entry, ..)) => Ok(entry),
            Err(e) => {
                log::error!("failed to get the object\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_objects(&self, prefix: EntryUrlPath, kind: Option<EntryKind>) -> Result<Vec<Entry>> {
        let url = prefix.to_url(ObjectsRepository::scheme());
        match self.objects_repository.list(url).await {
            Ok(mut entries) => {
                if let Some(kind) = kind {
                    entries.retain(|e| e.kind == kind);
                }
                Ok(entries)
            },
            Err(e) => {
                log::error!("failed to get objects\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn watch_medium_by_id(&self, id: MediumId, tag_depth: Option<TagDepth>, replicas: bool, sources: bool) -> Result<impl Stream<Item = Result<Medium>> + Send> {
        match self.media_repository.watch_by_id(id, tag_depth, replicas, sources).await {
            Ok(stream) => Ok(stream),
            Err(e) => {
                log::error!("failed to watch the medium\nError: {e:?}");
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
    ) -> Result<Medium>
    where
        T: CloneableIterator<Item = SourceId> + Send,
        U: CloneableIterator<Item = SourceId> + Send,
        V: CloneableIterator<Item = (TagId, TagTypeId)> + Send,
        W: CloneableIterator<Item = (TagId, TagTypeId)> + Send,
        X: CloneableIterator<Item = ReplicaId> + Send,
    {
        match self.media_repository.update_by_id(id, add_source_ids, remove_source_ids, add_tag_tag_type_ids, remove_tag_tag_type_ids, replica_orders, created_at, tag_depth, replicas, sources).await {
            Ok(medium) => Ok(medium),
            Err(e) => {
                log::error!("failed to update the medium\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn update_replica_by_id<R>(&self, id: ReplicaId, medium_source: MediumSource<R>) -> Result<(Replica, JoinHandle<()>)>
    where
        for<'a> R: Read + Seek + Send + 'a,
    {
        let (url, status, process) = self.create_replica_source(medium_source).await?;
        match self.replicas_repository.update_by_id(id, Some(None), Some(&url), Some(None), Some(ReplicaStatus::Processing)).await {
            Ok(replica) => {
                let handle = self.process_replica_by_id(replica.id, process);
                Ok((replica, handle))
            },
            Err(e) if status.is_created() => {
                log::error!("failed to update the replica\nError: {e:?}");

                if let Err(e) = self.objects_repository.delete(url).await {
                    log::error!("failed to delete the object\nError: {e:?}");
                }
                Err(e)
            },
            Err(e) => {
                log::error!("failed to update the replica\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn update_source_by_id(&self, id: SourceId, external_service_id: Option<ExternalServiceId>, external_metadata: Option<ExternalMetadata>) -> Result<Source> {
        match self.sources_repository.update_by_id(id, external_service_id, external_metadata).await {
            Ok(source) => Ok(source),
            Err(e) => {
                log::error!("failed to update the source\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn delete_medium_by_id(&self, id: MediumId, delete_objects: bool) -> Result<DeleteResult> {
        if delete_objects {
            let replicas = match self.media_repository.fetch_by_ids([id].into_iter(), None, true, false).await.map(|mut r| r.pop()) {
                Ok(Some(medium)) => medium.replicas,
                Ok(None) => return Ok(DeleteResult::NotFound),
                Err(e) => {
                    log::error!("failed to delete the objects of the media\nError: {e:?}");
                    return Err(e);
                },
            };

            for replica in replicas {
                if let Err(e) = self.objects_repository.delete(EntryUrl::from(replica.original_url)).await {
                    log::error!("failed to delete the objects of the media\nError: {e:?}");
                    return Err(e);
                }

                if let Err(e) = self.replicas_repository.delete_by_id(replica.id).await {
                    log::error!("failed to delete the replica of the media\nError: {e:?}");
                    return Err(e);
                }
            }
        }

        match self.media_repository.delete_by_id(id).await {
            Ok(result) => Ok(result),
            Err(e) => {
                log::error!("failed to delete the medium\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn delete_replica_by_id(&self, id: ReplicaId, delete_object: bool) -> Result<DeleteResult> {
        if delete_object {
            let replica = match self.replicas_repository.fetch_by_ids([id].into_iter()).await.map(|mut r| r.pop()) {
                Ok(Some(replica)) => replica,
                Ok(None) => return Ok(DeleteResult::NotFound),
                Err(e) => {
                    log::error!("failed to delete the object of the replica\nError: {e:?}");
                    return Err(e);
                },
            };

            if let Err(e) = self.objects_repository.delete(EntryUrl::from(replica.original_url)).await {
                log::error!("failed to delete the object of the replica\nError: {e:?}");
                return Err(e);
            }
        }

        match self.replicas_repository.delete_by_id(id).await {
            Ok(result) => Ok(result),
            Err(e) => {
                log::error!("failed to delete the replica\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn delete_source_by_id(&self, id: SourceId) -> Result<DeleteResult> {
        match self.sources_repository.delete_by_id(id).await {
            Ok(result) => Ok(result),
            Err(e) => {
                log::error!("failed to delete the source\nError: {e:?}");
                Err(e)
            },
        }
    }
}
