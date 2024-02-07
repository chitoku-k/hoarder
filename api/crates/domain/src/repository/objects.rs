use std::future::Future;

use tokio::io::{AsyncRead, AsyncSeek};

use crate::{
    entity::objects::Entry,
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

    #[cfg_attr(feature = "test-mock", mockall::concretize)]
    fn put<T>(&self, path: &str, content: T, overwrite: ObjectOverwriteBehavior) -> impl Future<Output = anyhow::Result<()>> + Send
    where
        T: AsyncRead + Send + Unpin;

    fn get(&self, path: &str) -> impl Future<Output = anyhow::Result<Self::Read>> + Send;

    fn list(&self, prefix: &str) -> impl Future<Output = anyhow::Result<Vec<Entry>>> + Send;

    fn delete(&self, path: &str) -> impl Future<Output = anyhow::Result<DeleteResult>> + Send;
}
