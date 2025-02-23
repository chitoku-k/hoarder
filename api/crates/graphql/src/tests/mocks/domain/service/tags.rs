use domain::{
    entity::{
        tag_types::{TagType, TagTypeId},
        tags::{Tag, TagDepth, TagId},
    },
    error::Result,
    iter::CloneableIterator,
    repository::{DeleteResult, Direction, Order},
    service::tags::TagsServiceInterface,
};

mockall::mock! {
    pub(crate) TagsServiceInterface {}

    impl TagsServiceInterface for TagsServiceInterface {
        #[mockall::concretize]
        fn create_tag<T>(&self, name: &str, kana: &str, aliases: T, parent_id: Option<TagId>, depth: TagDepth) -> impl Future<Output = Result<Tag>> + Send
        where
            T: CloneableIterator<Item = String> + Send;

        fn create_tag_type(&self, slug: &str, name: &str, kana: &str) -> impl Future<Output = Result<TagType>> + Send;

        fn get_tags(
            &self,
            depth: TagDepth,
            root: bool,
            cursor: Option<(String, TagId)>,
            order: Order,
            direction: Direction,
            limit: u64,
        ) -> impl Future<Output = Result<Vec<Tag>>> + Send;

        #[mockall::concretize]
        fn get_tags_by_ids<T>(&self, ids: T, depth: TagDepth) -> impl Future<Output = Result<Vec<Tag>>> + Send
        where
            T: CloneableIterator<Item = TagId> + Send;

        fn get_tags_by_name_or_alias_like(&self, name_or_alias_like: &str, depth: TagDepth) -> impl Future<Output = Result<Vec<Tag>>> + Send;

        fn get_tag_types(&self) -> impl Future<Output = Result<Vec<TagType>>> + Send;

        #[mockall::concretize]
        fn get_tag_types_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<TagType>>> + Send
        where
            T: CloneableIterator<Item = TagTypeId> + Send;

        #[mockall::concretize]
        fn update_tag_by_id<T, U>(&self, id: TagId, name: Option<String>, kana: Option<String>, add_aliases: T, remove_aliases: U, depth: TagDepth) -> impl Future<Output = Result<Tag>> + Send
        where
            T: CloneableIterator<Item = String> + Send,
            U: CloneableIterator<Item = String> + Send;

        fn update_tag_type_by_id<'a, 'b, 'c>(&self, id: TagTypeId, slug: Option<&'a str>, name: Option<&'b str>, kana: Option<&'c str>) -> impl Future<Output = Result<TagType>> + Send;

        fn attach_tag_by_id(&self, id: TagId, parent_id: TagId, depth: TagDepth) -> impl Future<Output = Result<Tag>> + Send;

        fn detach_tag_by_id(&self, id: TagId, depth: TagDepth) -> impl Future<Output = Result<Tag>> + Send;

        fn delete_tag_by_id(&self, id: TagId, recursive: bool) -> impl Future<Output = Result<DeleteResult>> + Send;

        fn delete_tag_type_by_id(&self, id: TagTypeId) -> impl Future<Output = Result<DeleteResult>> + Send;
    }
}
