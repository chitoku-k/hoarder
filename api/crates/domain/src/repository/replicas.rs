use async_trait::async_trait;

use crate::{
    entity::{
        media::MediumId,
        replicas::{Replica, ReplicaId, ReplicaThumbnail},
    },
    repository::DeleteResult,
};

#[cfg_attr(feature = "test-mock", mockall::automock)]
#[async_trait]
pub trait ReplicasRepository: Send + Sync + 'static {
    /// Creates a replica.
    async fn create(&self, medium_id: MediumId, thumbnail: Option<Vec<u8>>, original_url: &str, mime_type: &str) -> anyhow::Result<Replica>;

    /// Fetches the replicas by IDs.
    async fn fetch_by_ids<T>(&self, ids: T) -> anyhow::Result<Vec<Replica>>
    where
        T: IntoIterator<Item = ReplicaId> + Send + Sync + 'static;

    /// Fetches the replica by its original URL.
    async fn fetch_by_original_url(&self, original_url: &str) -> anyhow::Result<Replica>;

    /// Fetches the replica with thumbnail by ID.
    async fn fetch_thumbnail_by_id(&self, id: ReplicaId) -> anyhow::Result<ReplicaThumbnail>;

    /// Updates the replica.
    async fn update_by_id<'a>(&self, id: ReplicaId, thumbnail: Option<Vec<u8>>, original_url: Option<&'a str>, mime_type: Option<&'a str>) -> anyhow::Result<Replica>;

    /// Deletes the replica.
    async fn delete_by_id(&self, id: ReplicaId) -> anyhow::Result<DeleteResult>;
}
