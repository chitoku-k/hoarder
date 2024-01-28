use std::future::Future;

use crate::{
    entity::tag_types::{TagType, TagTypeId},
    repository::DeleteResult,
};

#[cfg_attr(feature = "test-mock", mockall::automock)]
pub trait TagTypesRepository: Send + Sync + 'static {
    /// Creates a tag type.
    fn create(&self, slug: &str, name: &str) -> impl Future<Output = anyhow::Result<TagType>> + Send;

    /// Fetches all tag types.
    fn fetch_all(&self) -> impl Future<Output = anyhow::Result<Vec<TagType>>> + Send;

    /// Updates the tag type by ID.
    fn update_by_id<'a>(&self, id: TagTypeId, slug: Option<&'a str>, name: Option<&'a str>) -> impl Future<Output = anyhow::Result<TagType>> + Send;

    /// Deletes the tag type by Id.
    fn delete_by_id(&self, id: TagTypeId) -> impl Future<Output = anyhow::Result<DeleteResult>> + Send;
}
