use crate::{
    entity::tags::{Tag, TagDepth, TagId},
    error::Result,
    iter::CloneableIterator,
    repository::{tags::TagsRepository, DeleteResult, Direction, Order},
};

mockall::mock! {
    pub(crate) TagsRepository {}

    impl TagsRepository for TagsRepository {
        #[mockall::concretize]
        fn create<T>(&self, name: &str, kana: &str, aliases: T, parent_id: Option<TagId>, depth: TagDepth) -> impl Future<Output = Result<Tag>> + Send
        where
            T: CloneableIterator<Item = String> + Send;

        #[mockall::concretize]
        fn fetch_by_ids<T>(&self, ids: T, depth: TagDepth) -> impl Future<Output = Result<Vec<Tag>>> + Send
        where
            T: CloneableIterator<Item = TagId> + Send;

        #[mockall::concretize]
        fn fetch_by_name_or_alias_like<T>(&self, query: T, depth: TagDepth) -> impl Future<Output = Result<Vec<Tag>>> + Send
        where
            T: CloneableIterator<Item = String> + Send;

        fn fetch_all(&self, depth: TagDepth, root: bool, cursor: Option<(String, TagId)>, order: Order, direction: Direction, limit: u64) -> impl Future<Output = Result<Vec<Tag>>> + Send;

        #[mockall::concretize]
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

        fn attach_by_id(&self, id: TagId, parent_id: TagId, depth: TagDepth) -> impl Future<Output = Result<Tag>> + Send;

        fn detach_by_id(&self, id: TagId, depth: TagDepth) -> impl Future<Output = Result<Tag>> + Send;

        fn delete_by_id(&self, id: TagId, recursive: bool) -> impl Future<Output = Result<DeleteResult>> + Send;
    }

    impl Clone for TagsRepository {
        fn clone(&self) -> Self;
    }
}
