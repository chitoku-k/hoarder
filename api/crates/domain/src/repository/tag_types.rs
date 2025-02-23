use crate::{
    entity::tag_types::{TagType, TagTypeId},
    error::Result,
    iter::CloneableIterator,
    repository::DeleteResult,
};

pub trait TagTypesRepository: Send + Sync + 'static {
    /// Creates a tag type.
    fn create(&self, slug: &str, name: &str, kana: &str) -> impl Future<Output = Result<TagType>> + Send;

    /// Fetches the tag types by their IDs.
    fn fetch_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<TagType>>> + Send
    where
        T: CloneableIterator<Item = TagTypeId> + Send;

    /// Fetches all tag types.
    fn fetch_all(&self) -> impl Future<Output = Result<Vec<TagType>>> + Send;

    /// Updates the tag type by ID.
    fn update_by_id(&self, id: TagTypeId, slug: Option<&str>, name: Option<&str>, kana: Option<&str>) -> impl Future<Output = Result<TagType>> + Send;

    /// Deletes the tag type by Id.
    fn delete_by_id(&self, id: TagTypeId) -> impl Future<Output = Result<DeleteResult>> + Send;
}
