use std::{future::Future, io::{Cursor, Read, Write}};

use domain::{
    entity::objects::{Entry, EntryUrl},
    error::Result,
    repository::{objects::{ObjectsRepository, ObjectOverwriteBehavior}, DeleteResult},
};

mockall::mock! {
    pub ObjectsRepository {}

    impl ObjectsRepository for ObjectsRepository {
        type Read = Cursor<&'static [u8]>;
        type Write = Cursor<Vec<u8>>;

        fn scheme() -> &'static str;

        fn put(&self, url: EntryUrl, overwrite: ObjectOverwriteBehavior) -> impl Future<Output = Result<(Entry, Cursor<Vec<u8>>)>> + Send;

        fn get(&self, url: EntryUrl) -> impl Future<Output = Result<(Entry, Cursor<&'static [u8]>)>> + Send;

        fn copy<R, W>(&self, read: &mut R, write: &mut W) -> Result<u64>
        where
            R: Read + 'static,
            W: Write + 'static;

        fn list(&self, prefix: EntryUrl) -> impl Future<Output = Result<Vec<Entry>>> + Send;

        fn delete(&self, url: EntryUrl) -> impl Future<Output = Result<DeleteResult>> + Send;
    }

    impl Clone for ObjectsRepository {
        fn clone(&self) -> Self;
    }
}
