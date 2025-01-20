use std::future::Future;

use domain::{
    entity::tags::{Tag, TagDepth, TagId},
    error::Result,
    repository::{tags::TagsRepository, DeleteResult, Direction, Order},
};

mockall::mock! {
    pub TagsRepository {}

    impl TagsRepository for TagsRepository {
        fn create<T>(&self, name: &str, kana: &str, aliases: T, parent_id: Option<TagId>, depth: TagDepth) -> impl Future<Output = Result<Tag>> + Send
        where
            T: IntoIterator<Item = String> + Send + 'static;

        fn fetch_by_ids<T>(&self, ids: T, depth: TagDepth) -> impl Future<Output = Result<Vec<Tag>>> + Send
        where
            T: IntoIterator<Item = TagId> + Send + 'static;

        fn fetch_by_name_or_alias_like(&self, name_or_alias_like: &str, depth: TagDepth) -> impl Future<Output = Result<Vec<Tag>>> + Send;

        fn fetch_all(&self, depth: TagDepth, root: bool, cursor: Option<(String, TagId)>, order: Order, direction: Direction, limit: u64) -> impl Future<Output = Result<Vec<Tag>>> + Send;

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
            T: IntoIterator<Item = String> + Send + 'static,
            U: IntoIterator<Item = String> + Send + 'static;

        fn attach_by_id(&self, id: TagId, parent_id: TagId, depth: TagDepth) -> impl Future<Output = Result<Tag>> + Send;

        fn detach_by_id(&self, id: TagId, depth: TagDepth) -> impl Future<Output = Result<Tag>> + Send;

        fn delete_by_id(&self, id: TagId, recursive: bool) -> impl Future<Output = Result<DeleteResult>> + Send;
    }

    impl Clone for TagsRepository {
        fn clone(&self) -> Self;
    }
}
