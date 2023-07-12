use async_trait::async_trait;

use crate::{
    entity::tags::{Tag, TagDepth, TagId},
    repository::{DeleteResult, OrderDirection},
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TagsRepository: Send + Sync + 'static {
    /// Creates a tag.
    async fn create(&self, name: &str, kana: &str, aliases: &[String], parent_id: Option<TagId>, depth: TagDepth) -> anyhow::Result<Tag>;

    /// Fetches tags by their IDs.
    async fn fetch_by_ids<T>(&self, ids: T, depth: TagDepth) -> anyhow::Result<Vec<Tag>>
    where
        T: IntoIterator<Item = TagId> + Send + Sync + 'static;

    /// Fetches tags by their names like the given parameter.
    async fn fetch_by_name_or_alias_like(&self, name_or_alias_like: &str, depth: TagDepth) -> anyhow::Result<Vec<Tag>>;

    /// Fetches all tags.
    async fn fetch_all(&self, depth: TagDepth, root: bool, after: Option<(String, TagId)>, before: Option<(String, TagId)>, order: OrderDirection, limit: u64) -> anyhow::Result<Vec<Tag>>;

    /// Updates the tag by ID.
    async fn update_by_id<T, U>(
        &self,
        id: TagId,
        name: Option<String>,
        kana: Option<String>,
        add_aliases: T,
        remove_aliases: U,
        depth: TagDepth,
    ) -> anyhow::Result<Tag>
    where
        T: IntoIterator<Item = String> + Send + Sync + 'static,
        U: IntoIterator<Item = String> + Send + Sync + 'static;

    /// Attaches the tag to the existing tag by ID.
    async fn attach_by_id(&self, id: TagId, parent_id: TagId, depth: TagDepth) -> anyhow::Result<Tag>;

    /// Detaches the tag from its parent by ID.
    async fn detach_by_id(&self, id: TagId, depth: TagDepth) -> anyhow::Result<Tag>;

    /// Deletes the tag by ID.
    async fn delete_by_id(&self, id: TagId, recursive: bool) -> anyhow::Result<DeleteResult>;
}
