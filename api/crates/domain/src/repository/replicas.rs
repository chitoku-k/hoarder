use crate::{
    entity::{
        media::MediumId,
        replicas::{OriginalImage, Replica, ReplicaId, ReplicaStatus, ThumbnailId, ThumbnailImage},
    },
    error::Result,
    iter::CloneableIterator,
    repository::DeleteResult,
};

pub trait ReplicasRepository: Send + Sync + 'static {
    /// Creates a replica.
    fn create(&self, medium_id: MediumId, thumbnail_image: Option<ThumbnailImage>, original_url: &str, original_image: Option<OriginalImage>, status: ReplicaStatus) -> impl Future<Output = Result<Replica>> + Send;

    /// Fetches the replicas by IDs.
    fn fetch_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<Replica>>> + Send
    where
        T: CloneableIterator<Item = ReplicaId> + Send;

    /// Fetches the replica by its original URL.
    fn fetch_by_original_url(&self, original_url: &str) -> impl Future<Output = Result<Replica>> + Send;

    /// Fetches the replica with thumbnail by ID.
    fn fetch_thumbnail_by_id(&self, id: ThumbnailId) -> impl Future<Output = Result<Vec<u8>>> + Send;

    /// Updates the replica.
    fn update_by_id(&self, id: ReplicaId, thumbnail_image: Option<Option<ThumbnailImage>>, original_url: Option<&str>, original_image: Option<Option<OriginalImage>>, status: Option<ReplicaStatus>) -> impl Future<Output = Result<Replica>> + Send;

    /// Deletes the replica.
    fn delete_by_id(&self, id: ReplicaId) -> impl Future<Output = Result<DeleteResult>> + Send;
}
