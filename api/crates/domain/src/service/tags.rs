use async_trait::async_trait;
use derive_more::Constructor;

use crate::{
    entity::{
        tag_types::{TagType, TagTypeId},
        tags::{Tag, TagDepth, TagId},
    },
    repository::{tag_types, tags, DeleteResult, Direction, Order},
};

#[cfg_attr(feature = "test-mock", mockall::automock)]
#[async_trait]
pub trait TagsServiceInterface: Send + Sync + 'static {
    /// Creates a tag.
    async fn create_tag(&self, name: &str, kana: &str, aliases: &[String], parent_id: Option<TagId>, depth: TagDepth) -> anyhow::Result<Tag>;

    /// Creates a tag type.
    async fn create_tag_type(&self, slug: &str, name: &str) -> anyhow::Result<TagType>;

    /// Gets tags.
    async fn get_tags(
        &self,
        depth: TagDepth,
        root: bool,
        cursor: Option<(String, TagId)>,
        order: Order,
        direction: Direction,
        limit: u64,
    ) -> anyhow::Result<Vec<Tag>>;

    /// Gets the tags by their IDs.
    async fn get_tags_by_ids<T>(&self, ids: T, depth: TagDepth) -> anyhow::Result<Vec<Tag>>
    where
        T: IntoIterator<Item = TagId> + Send + Sync + 'static;

    /// Gets the tags by their name or alias.
    async fn get_tags_by_name_or_alias_like(&self, name_or_alias_like: &str, depth: TagDepth) -> anyhow::Result<Vec<Tag>>;

    /// Gets tag types.
    async fn get_tag_types(&self) -> anyhow::Result<Vec<TagType>>;

    /// Updates the tag by ID.
    async fn update_tag_by_id<T, U>(&self, id: TagId, name: Option<String>, kana: Option<String>, add_aliases: T, remove_aliases: U, depth: TagDepth) -> anyhow::Result<Tag>
    where
        T: IntoIterator<Item = String> + Send + Sync + 'static,
        U: IntoIterator<Item = String> + Send + Sync + 'static;

    /// Updates the tag type by ID.
    async fn update_tag_type_by_id<'a, 'b>(&self, id: TagTypeId, slug: Option<&'a str>, name: Option<&'b str>) -> anyhow::Result<TagType>;

    /// Attaches the tag by ID.
    async fn attach_tag_by_id(&self, id: TagId, parent_id: TagId, depth: TagDepth) -> anyhow::Result<Tag>;

    /// Detaches the tag by ID.
    async fn detach_tag_by_id(&self, id: TagId, depth: TagDepth) -> anyhow::Result<Tag>;

    /// Delete the tag by ID.
    async fn delete_tag_by_id(&self, id: TagId, recursive: bool) -> anyhow::Result<DeleteResult>;

    /// Delete the tag type by ID.
    async fn delete_tag_type_by_id(&self, id: TagTypeId) -> anyhow::Result<DeleteResult>;
}

#[derive(Clone, Constructor)]
pub struct TagsService<TagsRepository, TagTypesRepository> {
    tags_repository: TagsRepository,
    tag_types_repository: TagTypesRepository,
}

#[async_trait]
impl<TagsRepository, TagTypesRepository> TagsServiceInterface for TagsService<TagsRepository, TagTypesRepository>
where
    TagsRepository: tags::TagsRepository,
    TagTypesRepository: tag_types::TagTypesRepository,
{
    async fn create_tag(&self, name: &str, kana: &str, aliases: &[String], parent_id: Option<TagId>, depth: TagDepth) -> anyhow::Result<Tag> {
        match self.tags_repository.create(name, kana, aliases, parent_id, depth).await {
            Ok(tag) => Ok(tag),
            Err(e) => {
                log::error!("failed to create a tag\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn create_tag_type(&self, slug: &str, name: &str) -> anyhow::Result<TagType> {
        match self.tag_types_repository.create(slug, name).await {
            Ok(tag_type) => Ok(tag_type),
            Err(e) => {
                log::error!("failed to create a tag type\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_tags(
        &self,
        depth: TagDepth,
        root: bool,
        cursor: Option<(String, TagId)>,
        order: Order,
        direction: Direction,
        limit: u64,
    ) -> anyhow::Result<Vec<Tag>> {
        match self.tags_repository.fetch_all(depth, root, cursor, order, direction, limit).await {
            Ok(tags) => Ok(tags),
            Err(e) => {
                log::error!("failed to get tags\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_tags_by_ids<T>(&self, ids: T, depth: TagDepth) -> anyhow::Result<Vec<Tag>>
    where
        T: IntoIterator<Item = TagId> + Send + Sync + 'static,
    {
        match self.tags_repository.fetch_by_ids(ids, depth).await {
            Ok(tags) => Ok(tags),
            Err(e) => {
                log::error!("failed to get tags\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_tags_by_name_or_alias_like(&self, name_or_alias_like: &str, depth: TagDepth) -> anyhow::Result<Vec<Tag>> {
        match self.tags_repository.fetch_by_name_or_alias_like(name_or_alias_like, depth).await {
            Ok(tags) => Ok(tags),
            Err(e) => {
                log::error!("failed to get tags\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn get_tag_types(&self) -> anyhow::Result<Vec<TagType>> {
        match self.tag_types_repository.fetch_all().await {
            Ok(tag_types) => Ok(tag_types),
            Err(e) => {
                log::error!("failed to get tag types\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn update_tag_by_id<T, U>(&self, id: TagId, name: Option<String>, kana: Option<String>, add_aliases: T, remove_aliases: U, depth: TagDepth) -> anyhow::Result<Tag>
    where
        T: IntoIterator<Item = String> + Send + Sync + 'static,
        U: IntoIterator<Item = String> + Send + Sync + 'static,
    {
        match self.tags_repository.update_by_id(id, name, kana, add_aliases, remove_aliases, depth).await {
            Ok(tag) => Ok(tag),
            Err(e) => {
                log::error!("failed to update the tag\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn update_tag_type_by_id<'a, 'b>(&self, id: TagTypeId, slug: Option<&'a str>, name: Option<&'b str>) -> anyhow::Result<TagType> {
        match self.tag_types_repository.update_by_id(id, slug, name).await {
            Ok(tag_type) => Ok(tag_type),
            Err(e) => {
                log::error!("failed to update the tag type\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn attach_tag_by_id(&self, id: TagId, parent_id: TagId, depth: TagDepth) -> anyhow::Result<Tag> {
        match self.tags_repository.attach_by_id(id, parent_id, depth).await {
            Ok(tag) => Ok(tag),
            Err(e) => {
                log::error!("failed to attach the tag\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn detach_tag_by_id(&self, id: TagId, depth: TagDepth) -> anyhow::Result<Tag> {
        match self.tags_repository.detach_by_id(id, depth).await {
            Ok(tag) => Ok(tag),
            Err(e) => {
                log::error!("failed to detach the tag\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn delete_tag_by_id(&self, id: TagId, recursive: bool) -> anyhow::Result<DeleteResult> {
        match self.tags_repository.delete_by_id(id, recursive).await {
            Ok(result) => Ok(result),
            Err(e) => {
                log::error!("failed to delete the tag\nError: {e:?}");
                Err(e)
            },
        }
    }

    async fn delete_tag_type_by_id(&self, id: TagTypeId) -> anyhow::Result<DeleteResult> {
        match self.tag_types_repository.delete_by_id(id).await {
            Ok(result) => Ok(result),
            Err(e) => {
                log::error!("failed to delete the tag type\nError: {e:?}");
                Err(e)
            },
        }
    }
}
