use std::future::Future;

use crate::{
    entity::{
        media::MediumId,
        replicas::{OriginalImage, Replica, ReplicaId, ThumbnailId, ThumbnailImage},
    },
    error::Result,
    repository::DeleteResult,
};

#[cfg_attr(feature = "test-mock", mockall::automock)]
pub trait ReplicasRepository: Send + Sync + 'static {
    /// Creates a replica.
    fn create(&self, medium_id: MediumId, thumbnail_image: Option<ThumbnailImage>, original_url: &str, original_image: OriginalImage) -> impl Future<Output = Result<Replica>> + Send;

    /// Fetches the replicas by IDs.
    fn fetch_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<Replica>>> + Send
    where
        T: IntoIterator<Item = ReplicaId> + Send + Sync + 'static;

    /// Fetches the replica by its original URL.
    fn fetch_by_original_url(&self, original_url: &str) -> impl Future<Output = Result<Replica>> + Send;

    /// Fetches the replica with thumbnail by ID.
    fn fetch_thumbnail_by_id(&self, id: ThumbnailId) -> impl Future<Output = Result<Vec<u8>>> + Send;

    /// Updates the replica.
    fn update_by_id<'a>(&self, id: ReplicaId, thumbnail_image: Option<ThumbnailImage>, original_url: Option<&'a str>, original_image: Option<OriginalImage>) -> impl Future<Output = Result<Replica>> + Send;

    /// Deletes the replica.
    fn delete_by_id(&self, id: ReplicaId) -> impl Future<Output = Result<DeleteResult>> + Send;
}
