use derive_more::Constructor;

use crate::{
    entity::{
        tag_types::{TagType, TagTypeId},
        tags::{Tag, TagDepth, TagId},
    },
    error::Result,
    iter::CloneableIterator,
    repository::{tag_types, tags, DeleteResult, Direction, Order},
};

pub trait TagsServiceInterface: Send + Sync + 'static {
    /// Creates a tag.
    fn create_tag<T>(&self, name: &str, kana: &str, aliases: T, parent_id: Option<TagId>, depth: TagDepth) -> impl Future<Output = Result<Tag>> + Send
    where
        T: CloneableIterator<Item = String> + Send;

    /// Creates a tag type.
    fn create_tag_type(&self, slug: &str, name: &str, kana: &str) -> impl Future<Output = Result<TagType>> + Send;

    /// Gets tags.
    fn get_tags(
        &self,
        depth: TagDepth,
        root: bool,
        cursor: Option<(String, TagId)>,
        order: Order,
        direction: Direction,
        limit: u64,
    ) -> impl Future<Output = Result<Vec<Tag>>> + Send;

    /// Gets the tags by their IDs.
    fn get_tags_by_ids<T>(&self, ids: T, depth: TagDepth) -> impl Future<Output = Result<Vec<Tag>>> + Send
    where
        T: CloneableIterator<Item = TagId> + Send;

    /// Gets the tags by their name or alias.
    fn get_tags_by_name_or_alias_like(&self, name_or_alias_like: &str, depth: TagDepth) -> impl Future<Output = Result<Vec<Tag>>> + Send;

    /// Gets tag types.
    fn get_tag_types(&self) -> impl Future<Output = Result<Vec<TagType>>> + Send;

    /// Gets the tag types by their IDs.
    fn get_tag_types_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<TagType>>> + Send
    where
        T: CloneableIterator<Item = TagTypeId> + Send;

    /// Updates the tag by ID.
    fn update_tag_by_id<T, U>(&self, id: TagId, name: Option<String>, kana: Option<String>, add_aliases: T, remove_aliases: U, depth: TagDepth) -> impl Future<Output = Result<Tag>> + Send
    where
        T: CloneableIterator<Item = String> + Send,
        U: CloneableIterator<Item = String> + Send;

    /// Updates the tag type by ID.
    fn update_tag_type_by_id(&self, id: TagTypeId, slug: Option<&str>, name: Option<&str>, kana: Option<&str>) -> impl Future<Output = Result<TagType>> + Send;

    /// Attaches the tag by ID.
    fn attach_tag_by_id(&self, id: TagId, parent_id: TagId, depth: TagDepth) -> impl Future<Output = Result<Tag>> + Send;

    /// Detaches the tag by ID.
    fn detach_tag_by_id(&self, id: TagId, depth: TagDepth) -> impl Future<Output = Result<Tag>> + Send;

    /// Delete the tag by ID.
    fn delete_tag_by_id(&self, id: TagId, recursive: bool) -> impl Future<Output = Result<DeleteResult>> + Send;

    /// Delete the tag type by ID.
    fn delete_tag_type_by_id(&self, id: TagTypeId) -> impl Future<Output = Result<DeleteResult>> + Send;
}

#[derive(Clone, Constructor)]
pub struct TagsService<TagsRepository, TagTypesRepository> {
    tags_repository: TagsRepository,
    tag_types_repository: TagTypesRepository,
}

impl<TagsRepository, TagTypesRepository> TagsServiceInterface for TagsService<TagsRepository, TagTypesRepository>
where
    TagsRepository: tags::TagsRepository,
    TagTypesRepository: tag_types::TagTypesRepository,
{
    #[tracing::instrument(skip_all)]
    async fn create_tag<T>(&self, name: &str, kana: &str, aliases: T, parent_id: Option<TagId>, depth: TagDepth) -> Result<Tag>
    where
        T: CloneableIterator<Item = String> + Send,
    {
        match self.tags_repository.create(name, kana, aliases, parent_id, depth).await {
            Ok(tag) => Ok(tag),
            Err(e) => {
                tracing::error!("failed to create a tag\nError: {e:?}");
                Err(e)
            },
        }
    }

    #[tracing::instrument(skip_all)]
    async fn create_tag_type(&self, slug: &str, name: &str, kana: &str) -> Result<TagType> {
        match self.tag_types_repository.create(slug, name, kana).await {
            Ok(tag_type) => Ok(tag_type),
            Err(e) => {
                tracing::error!("failed to create a tag type\nError: {e:?}");
                Err(e)
            },
        }
    }

    #[tracing::instrument(skip_all)]
    async fn get_tags(
        &self,
        depth: TagDepth,
        root: bool,
        cursor: Option<(String, TagId)>,
        order: Order,
        direction: Direction,
        limit: u64,
    ) -> Result<Vec<Tag>> {
        match self.tags_repository.fetch_all(depth, root, cursor, order, direction, limit).await {
            Ok(tags) => Ok(tags),
            Err(e) => {
                tracing::error!("failed to get the tags\nError: {e:?}");
                Err(e)
            },
        }
    }

    #[tracing::instrument(skip_all)]
    async fn get_tags_by_ids<T>(&self, ids: T, depth: TagDepth) -> Result<Vec<Tag>>
    where
        T: CloneableIterator<Item = TagId> + Send,
    {
        match self.tags_repository.fetch_by_ids(ids, depth).await {
            Ok(tags) => Ok(tags),
            Err(e) => {
                tracing::error!("failed to get the tags\nError: {e:?}");
                Err(e)
            },
        }
    }

    #[tracing::instrument(skip_all)]
    async fn get_tags_by_name_or_alias_like(&self, name_or_alias_like: &str, depth: TagDepth) -> Result<Vec<Tag>> {
        match self.tags_repository.fetch_by_name_or_alias_like(name_or_alias_like, depth).await {
            Ok(tags) => Ok(tags),
            Err(e) => {
                tracing::error!("failed to get the tags\nError: {e:?}");
                Err(e)
            },
        }
    }

    #[tracing::instrument(skip_all)]
    async fn get_tag_types(&self) -> Result<Vec<TagType>> {
        match self.tag_types_repository.fetch_all().await {
            Ok(tag_types) => Ok(tag_types),
            Err(e) => {
                tracing::error!("failed to get the tag types\nError: {e:?}");
                Err(e)
            },
        }
    }

    #[tracing::instrument(skip_all)]
    async fn get_tag_types_by_ids<T>(&self, ids: T) -> Result<Vec<TagType>>
    where
        T: CloneableIterator<Item = TagTypeId> + Send,
    {
        match self.tag_types_repository.fetch_by_ids(ids).await {
            Ok(tag_types) => Ok(tag_types),
            Err(e) => {
                tracing::error!("failed to get the tag types\nError: {e:?}");
                Err(e)
            },
        }
    }

    #[tracing::instrument(skip_all)]
    async fn update_tag_by_id<T, U>(&self, id: TagId, name: Option<String>, kana: Option<String>, add_aliases: T, remove_aliases: U, depth: TagDepth) -> Result<Tag>
    where
        T: CloneableIterator<Item = String> + Send,
        U: CloneableIterator<Item = String> + Send,
    {
        match self.tags_repository.update_by_id(id, name, kana, add_aliases, remove_aliases, depth).await {
            Ok(tag) => Ok(tag),
            Err(e) => {
                tracing::error!("failed to update the tag\nError: {e:?}");
                Err(e)
            },
        }
    }

    #[tracing::instrument(skip_all)]
    async fn update_tag_type_by_id(&self, id: TagTypeId, slug: Option<&str>, name: Option<&str>, kana: Option<&str>) -> Result<TagType> {
        match self.tag_types_repository.update_by_id(id, slug, name, kana).await {
            Ok(tag_type) => Ok(tag_type),
            Err(e) => {
                tracing::error!("failed to update the tag type\nError: {e:?}");
                Err(e)
            },
        }
    }

    #[tracing::instrument(skip_all)]
    async fn attach_tag_by_id(&self, id: TagId, parent_id: TagId, depth: TagDepth) -> Result<Tag> {
        match self.tags_repository.attach_by_id(id, parent_id, depth).await {
            Ok(tag) => Ok(tag),
            Err(e) => {
                tracing::error!("failed to attach the tag\nError: {e:?}");
                Err(e)
            },
        }
    }

    #[tracing::instrument(skip_all)]
    async fn detach_tag_by_id(&self, id: TagId, depth: TagDepth) -> Result<Tag> {
        match self.tags_repository.detach_by_id(id, depth).await {
            Ok(tag) => Ok(tag),
            Err(e) => {
                tracing::error!("failed to detach the tag\nError: {e:?}");
                Err(e)
            },
        }
    }

    #[tracing::instrument(skip_all)]
    async fn delete_tag_by_id(&self, id: TagId, recursive: bool) -> Result<DeleteResult> {
        match self.tags_repository.delete_by_id(id, recursive).await {
            Ok(result) => Ok(result),
            Err(e) => {
                tracing::error!("failed to delete the tag\nError: {e:?}");
                Err(e)
            },
        }
    }

    #[tracing::instrument(skip_all)]
    async fn delete_tag_type_by_id(&self, id: TagTypeId) -> Result<DeleteResult> {
        match self.tag_types_repository.delete_by_id(id).await {
            Ok(result) => Ok(result),
            Err(e) => {
                tracing::error!("failed to delete the tag type\nError: {e:?}");
                Err(e)
            },
        }
    }
}
