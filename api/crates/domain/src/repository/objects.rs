use std::future::Future;

use tokio::io::{AsyncRead, AsyncSeek, AsyncWrite};

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
    type Read: AsyncRead + AsyncSeek + Send + Unpin + 'static;
    type Write: AsyncWrite + AsyncSeek + Send + Unpin + 'static;

    fn scheme() -> &'static str;

    fn put(&self, url: EntryUrl, overwrite: ObjectOverwriteBehavior) -> impl Future<Output = Result<(Entry, Self::Write)>> + Send;

    fn get(&self, url: EntryUrl) -> impl Future<Output = Result<(Entry, Self::Read)>> + Send;

    fn list(&self, prefix: EntryUrl) -> impl Future<Output = Result<Vec<Entry>>> + Send;

    fn delete(&self, url: EntryUrl) -> impl Future<Output = Result<DeleteResult>> + Send;
}
