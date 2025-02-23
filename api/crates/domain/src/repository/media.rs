use chrono::{DateTime, Utc};
use futures::Stream;

use crate::{
    entity::{
        media::{Medium, MediumId},
        replicas::ReplicaId,
        sources::SourceId,
        tag_types::TagTypeId,
        tags::{TagDepth, TagId},
    },
    iter::CloneableIterator,
    error::Result,
    repository::{DeleteResult, Direction, Order},
};

pub trait MediaRepository: Send + Sync + 'static {
    /// Creates a medium.
    fn create<T, U>(&self, source_ids: T, created_at: Option<DateTime<Utc>>, tag_tag_type_ids: U, tag_depth: Option<TagDepth>, sources: bool) -> impl Future<Output = Result<Medium>> + Send
    where
        T: CloneableIterator<Item = SourceId> + Send,
        U: CloneableIterator<Item = (TagId, TagTypeId)> + Send;

    /// Fetches media by IDs.
    fn fetch_by_ids<T>(&self, ids: T, tag_depth: Option<TagDepth>, replicas: bool, sources: bool) -> impl Future<Output = Result<Vec<Medium>>> + Send
    where
        T: CloneableIterator<Item = MediumId> + Send;

    /// Fetches media by their associated source IDs.
    fn fetch_by_source_ids<T>(
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

    /// Fetches media by their associated tag IDs.
    fn fetch_by_tag_ids<T>(
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

    /// Fetches all media.
    fn fetch_all(
        &self,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
        cursor: Option<(DateTime<Utc>, MediumId)>,
        order: Order,
        direction: Direction,
        limit: u64,
    ) -> impl Future<Output = Result<Vec<Medium>>> + Send;

    /// Watches the medium by ID.
    fn watch_by_id(&self, id: MediumId, tag_depth: Option<TagDepth>, replicas: bool, sources: bool) -> impl Future<Output = Result<impl Stream<Item = Result<Medium>> + Send>> + Send;

    /// Updates the medium by ID.
    fn update_by_id<T, U, V, W, X>(
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

    /// Deletes the medium by ID.
    fn delete_by_id(&self, id: MediumId) -> impl Future<Output = Result<DeleteResult>> + Send;
}
