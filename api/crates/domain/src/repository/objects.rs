use std::{future::Future, io::{Read, Seek, Write}};

use crate::{
    entity::objects::{Entry, EntryUrl},
    error::Result,
    repository::DeleteResult,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectOverwriteBehavior {
    Overwrite,
    Fail,
}

impl ObjectOverwriteBehavior {
    pub const fn is_allowed(&self) -> bool {
        matches!(self, ObjectOverwriteBehavior::Overwrite)
    }

    pub const fn is_denied(&self) -> bool {
        matches!(self, ObjectOverwriteBehavior::Fail)
    }
}

pub trait ObjectsRepository: Send + Sync + 'static {
    type Read: Read + Seek + Send + Unpin + 'static;
    type Write: Write + Seek + Send + Unpin + 'static;

    fn scheme() -> &'static str;

    fn put(&self, url: EntryUrl, overwrite: ObjectOverwriteBehavior) -> impl Future<Output = Result<(Entry, Self::Write)>> + Send;

    fn get(&self, url: EntryUrl) -> impl Future<Output = Result<(Entry, Self::Read)>> + Send;

    fn copy<R>(&self, read: &mut R, write: &mut Self::Write) -> Result<u64>
    where
        for<'a> R: Read + 'a;

    fn list(&self, prefix: EntryUrl) -> impl Future<Output = Result<Vec<Entry>>> + Send;

    fn delete(&self, url: EntryUrl) -> impl Future<Output = Result<DeleteResult>> + Send;
}
