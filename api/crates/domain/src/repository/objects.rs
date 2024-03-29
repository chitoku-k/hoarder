use std::future::Future;

use tokio::io::{AsyncRead, AsyncSeek};

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
    pub fn is_allowed(&self) -> bool {
        matches!(self, ObjectOverwriteBehavior::Overwrite)
    }

    pub fn is_denied(&self) -> bool {
        matches!(self, ObjectOverwriteBehavior::Fail)
    }
}

#[cfg_attr(feature = "test-mock", mockall::automock(type Read = std::io::Cursor<&'static [u8]>;))]
pub trait ObjectsRepository: Send + Sync + 'static {
    type Read: AsyncRead + AsyncSeek + Send + Unpin + 'static;

    fn scheme() -> &'static str;

    #[cfg_attr(feature = "test-mock", mockall::concretize)]
    fn put<T>(&self, url: EntryUrl, content: T, overwrite: ObjectOverwriteBehavior) -> impl Future<Output = Result<Entry>> + Send
    where
        T: AsyncRead + Send + Unpin;

    fn get(&self, url: EntryUrl) -> impl Future<Output = Result<(Entry, Self::Read)>> + Send;

    fn list(&self, prefix: EntryUrl) -> impl Future<Output = Result<Vec<Entry>>> + Send;

    fn delete(&self, url: EntryUrl) -> impl Future<Output = Result<DeleteResult>> + Send;
}
