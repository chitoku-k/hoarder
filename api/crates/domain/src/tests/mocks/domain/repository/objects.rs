use std::io::Cursor;

use tokio::io::AsyncRead;

use crate::{
    entity::objects::{Entry, EntryUrl},
    error::Result,
    repository::{objects::{ObjectOverwriteBehavior, ObjectStatus, ObjectsRepository}, DeleteResult},
};

mockall::mock! {
    pub(crate) ObjectsRepository {}

    impl ObjectsRepository for ObjectsRepository {
        type Put = Vec<u8>;
        type Get = Cursor<&'static [u8]>;

        fn scheme() -> &'static str;

        fn put(&self, url: EntryUrl, overwrite: ObjectOverwriteBehavior) -> impl Future<Output = Result<(Entry, ObjectStatus, Vec<u8>)>> + Send;

        fn get(&self, url: EntryUrl) -> impl Future<Output = Result<(Entry, Cursor<&'static [u8]>)>> + Send;

        fn entry(&self, url: EntryUrl) -> impl Future<Output = Result<Entry>> + Send;

        fn copy<R>(&self, read: &mut R, write: &mut Vec<u8>) -> impl Future<Output = Result<u64>> + Send
        where
            R: AsyncRead + Send + Unpin + 'static;

        fn list(&self, prefix: EntryUrl) -> impl Future<Output = Result<Vec<Entry>>> + Send;

        fn delete(&self, url: EntryUrl) -> impl Future<Output = Result<DeleteResult>> + Send;
    }

    impl Clone for ObjectsRepository {
        fn clone(&self) -> Self;
    }
}
