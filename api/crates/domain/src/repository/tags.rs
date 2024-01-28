use std::future::Future;

use crate::{
    entity::tags::{Tag, TagDepth, TagId},
    repository::{DeleteResult, Direction, Order},
};

#[cfg_attr(feature = "test-mock", mockall::automock)]
pub trait TagsRepository: Send + Sync + 'static {
    /// Creates a tag.
    fn create(&self, name: &str, kana: &str, aliases: &[String], parent_id: Option<TagId>, depth: TagDepth) -> impl Future<Output = anyhow::Result<Tag>> + Send;

    /// Fetches tags by their IDs.
    fn fetch_by_ids<T>(&self, ids: T, depth: TagDepth) -> impl Future<Output = anyhow::Result<Vec<Tag>>> + Send
    where
        T: IntoIterator<Item = TagId> + Send + Sync + 'static;

    /// Fetches tags by their names like the given parameter.
    fn fetch_by_name_or_alias_like(&self, name_or_alias_like: &str, depth: TagDepth) -> impl Future<Output = anyhow::Result<Vec<Tag>>> + Send;

    /// Fetches all tags.
    fn fetch_all(&self, depth: TagDepth, root: bool, cursor: Option<(String, TagId)>, order: Order, direction: Direction, limit: u64) -> impl Future<Output = anyhow::Result<Vec<Tag>>> + Send;

    /// Updates the tag by ID.
    fn update_by_id<T, U>(
        &self,
        id: TagId,
        name: Option<String>,
        kana: Option<String>,
        add_aliases: T,
        remove_aliases: U,
        depth: TagDepth,
    ) -> impl Future<Output = anyhow::Result<Tag>> + Send
    where
        T: IntoIterator<Item = String> + Send + Sync + 'static,
        U: IntoIterator<Item = String> + Send + Sync + 'static;

    /// Attaches the tag to the existing tag by ID.
    fn attach_by_id(&self, id: TagId, parent_id: TagId, depth: TagDepth) -> impl Future<Output = anyhow::Result<Tag>> + Send;

    /// Detaches the tag from its parent by ID.
    fn detach_by_id(&self, id: TagId, depth: TagDepth) -> impl Future<Output = anyhow::Result<Tag>> + Send;

    /// Deletes the tag by ID.
    fn delete_by_id(&self, id: TagId, recursive: bool) -> impl Future<Output = anyhow::Result<DeleteResult>> + Send;
}
