use std::future::Future;

use domain::{
    entity::tag_types::{TagType, TagTypeId},
    error::Result,
    repository::{tag_types::TagTypesRepository, DeleteResult},
};

mockall::mock! {
    pub TagTypesRepository {}

    impl TagTypesRepository for TagTypesRepository {
        fn create(&self, slug: &str, name: &str, kana: &str) -> impl Future<Output = Result<TagType>> + Send;

        fn fetch_by_ids<T>(&self, ids: T) -> impl Future<Output = Result<Vec<TagType>>> + Send
        where
            T: IntoIterator<Item = TagTypeId> + Send + 'static;

        fn fetch_all(&self) -> impl Future<Output = Result<Vec<TagType>>> + Send;

        fn update_by_id<'a>(&self, id: TagTypeId, slug: Option<&'a str>, name: Option<&'a str>, kana: Option<&'a str>) -> impl Future<Output = Result<TagType>> + Send;

        fn delete_by_id(&self, id: TagTypeId) -> impl Future<Output = Result<DeleteResult>> + Send;
    }
}
