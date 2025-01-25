use std::{future::Future, io::{Cursor, Read}};

use domain::{
    entity::objects::{Entry, EntryUrl},
    error::Result,
    repository::{objects::{ObjectOverwriteBehavior, ObjectStatus, ObjectsRepository}, DeleteResult},
};

mockall::mock! {
    pub ObjectsRepository {}

    impl ObjectsRepository for ObjectsRepository {
        type Read = Cursor<&'static [u8]>;
        type Write = Cursor<Vec<u8>>;

        fn scheme() -> &'static str;

        fn put(&self, url: EntryUrl, overwrite: ObjectOverwriteBehavior) -> impl Future<Output = Result<(Entry, ObjectStatus, Cursor<Vec<u8>>)>> + Send;

        fn get(&self, url: EntryUrl) -> impl Future<Output = Result<(Entry, Cursor<&'static [u8]>)>> + Send;

        fn copy<R>(&self, read: &mut R, write: &mut Cursor<Vec<u8>>) -> Result<u64>
        where
            R: Read + 'static;

        fn list(&self, prefix: EntryUrl) -> impl Future<Output = Result<Vec<Entry>>> + Send;

        fn delete(&self, url: EntryUrl) -> impl Future<Output = Result<DeleteResult>> + Send;
    }

    impl Clone for ObjectsRepository {
        fn clone(&self) -> Self;
    }
}
