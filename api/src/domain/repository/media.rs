use async_trait::async_trait;
use chrono::NaiveDateTime;

use crate::domain::{
    entity::{
        media::{Medium, MediumId},
        replicas::ReplicaId,
        sources::SourceId,
        tag_types::TagTypeId,
        tags::{TagDepth, TagId},
    },
    repository::{DeleteResult, OrderDirection},
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MediaRepository: Send + Sync + 'static {
    /// Creates a medium.
    async fn create(
        &self,
        source_ids: Vec<SourceId>,
        created_at: Option<NaiveDateTime>,
        tag_tag_type_ids: Vec<(TagId, TagTypeId)>,
        tag_depth: Option<TagDepth>,
        sources: bool,
    ) -> anyhow::Result<Medium>;

    /// Fetches media by IDs.
    async fn fetch_by_ids<T>(&self, ids: T, tag_depth: Option<TagDepth>, replicas: bool, sources: bool) -> anyhow::Result<Vec<Medium>>
    where
        T: IntoIterator<Item = MediumId> + Send + Sync + 'static;

    /// Fetches media by their associated source IDs.
    async fn fetch_by_source_ids<T>(
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

    /// Fetches media by their associated tag IDs.
    async fn fetch_by_tag_ids<T>(
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

    /// Fetches all media.
    async fn fetch_all(
        &self,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
        since: Option<(NaiveDateTime, MediumId)>,
        until: Option<(NaiveDateTime, MediumId)>,
        order: OrderDirection,
        limit: u64,
    ) -> anyhow::Result<Vec<Medium>>;

    /// Updates the medium by ID.
    async fn update_by_id<T>(
        &self,
        id: MediumId,
        add_source_ids: Vec<SourceId>,
        remove_source_ids: Vec<SourceId>,
        add_tag_tag_type_ids: Vec<(TagId, TagTypeId)>,
        remove_tag_tag_type_ids: Vec<(TagId, TagTypeId)>,
        replica_orders: T,
        created_at: Option<NaiveDateTime>,
        tag_depth: Option<TagDepth>,
        replicas: bool,
        sources: bool,
    ) -> anyhow::Result<Medium>
    where
        T: IntoIterator<Item = ReplicaId> + Send + Sync + 'static;

    /// Deletes the medium by ID.
    async fn delete_by_id(&self, id: MediumId) -> anyhow::Result<DeleteResult>;
}
