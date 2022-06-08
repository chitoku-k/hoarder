use async_trait::async_trait;

use crate::domain::{
    entity::tag_types::{TagType, TagTypeId},
    repository::DeleteResult,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TagTypesRepository: Send + Sync + 'static {
    /// Creates a tag type.
    async fn create(&self, slug: &'_ str, name: &'_ str) -> anyhow::Result<TagType>;

    /// Fetches all tag types.
    async fn fetch_all(&self) -> anyhow::Result<Vec<TagType>>;

    /// Updates the tag type by ID.
    async fn update_by_id<'a>(&self, id: TagTypeId, slug: Option<&'a str>, name: Option<&'a str>) -> anyhow::Result<TagType>;

    /// Deletes the tag type by Id.
    async fn delete_by_id(&self, id: TagTypeId) -> anyhow::Result<DeleteResult>;
}
