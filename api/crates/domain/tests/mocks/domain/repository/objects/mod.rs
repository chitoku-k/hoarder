use std::{future::Future, io::Cursor};

use domain::{
    entity::objects::{Entry, EntryUrl},
    error::Result,
    repository::{objects::{ObjectsRepository, ObjectOverwriteBehavior}, DeleteResult},
};
use tokio::io::AsyncRead;

mockall::mock! {
    pub ObjectsRepository {}

    impl ObjectsRepository for ObjectsRepository {
        type Read = Cursor<&'static [u8]>;

        fn scheme() -> &'static str;

        #[mockall::concretize]
        fn put<T>(&self, url: EntryUrl, content: T, overwrite: ObjectOverwriteBehavior) -> impl Future<Output = Result<Entry>> + Send
        where
            T: AsyncRead + Send + Unpin;

        fn get(&self, url: EntryUrl) -> impl Future<Output = Result<(Entry, Cursor<&'static [u8]>)>> + Send;

        fn list(&self, prefix: EntryUrl) -> impl Future<Output = Result<Vec<Entry>>> + Send;

        fn delete(&self, url: EntryUrl) -> impl Future<Output = Result<DeleteResult>> + Send;
    }

    impl Clone for ObjectsRepository {
        fn clone(&self) -> Self;
    }
}
