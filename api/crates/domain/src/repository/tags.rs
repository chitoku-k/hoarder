use crate::{
    entity::tags::{Tag, TagDepth, TagId},
    error::Result,
    iter::CloneableIterator,
    repository::{DeleteResult, Direction, Order},
};

pub trait TagsRepository: Send + Sync + 'static {
    /// Creates a tag.
    fn create<T>(&self, name: &str, kana: &str, aliases: T, parent_id: Option<TagId>, depth: TagDepth) -> impl Future<Output = Result<Tag>> + Send
    where
        T: CloneableIterator<Item = String> + Send;

    /// Fetches tags by their IDs.
    fn fetch_by_ids<T>(&self, ids: T, depth: TagDepth) -> impl Future<Output = Result<Vec<Tag>>> + Send
    where
        T: CloneableIterator<Item = TagId> + Send;

    /// Fetches tags by their names like the given parameter.
    fn fetch_by_name_or_alias_like<T>(&self, query: T, depth: TagDepth) -> impl Future<Output = Result<Vec<Tag>>> + Send
    where
        T: CloneableIterator<Item = String> + Send;

    /// Fetches all tags.
    fn fetch_all(&self, depth: TagDepth, root: bool, cursor: Option<(String, TagId)>, order: Order, direction: Direction, limit: u64) -> impl Future<Output = Result<Vec<Tag>>> + Send;

    /// Updates the tag by ID.
    fn update_by_id<T, U>(
        &self,
        id: TagId,
        name: Option<String>,
        kana: Option<String>,
        add_aliases: T,
        remove_aliases: U,
        depth: TagDepth,
    ) -> impl Future<Output = Result<Tag>> + Send
    where
        T: CloneableIterator<Item = String> + Send,
        U: CloneableIterator<Item = String> + Send;

    /// Attaches the tag to the existing tag by ID.
    fn attach_by_id(&self, id: TagId, parent_id: TagId, depth: TagDepth) -> impl Future<Output = Result<Tag>> + Send;

    /// Detaches the tag from its parent by ID.
    fn detach_by_id(&self, id: TagId, depth: TagDepth) -> impl Future<Output = Result<Tag>> + Send;

    /// Deletes the tag by ID.
    fn delete_by_id(&self, id: TagId, recursive: bool) -> impl Future<Output = Result<DeleteResult>> + Send;
}
