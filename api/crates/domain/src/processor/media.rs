use std::{future::Future, io::{BufRead, Seek}};

use crate::entity::replicas::{OriginalImage, ThumbnailImage};

#[cfg_attr(feature = "test-mock", mockall::automock)]
pub trait MediumImageProcessor: Send + Sync + 'static {
    /// Generates a thumbnail for image on the given path.
    fn generate_thumbnail<R>(&self, read: R) -> impl Future<Output = anyhow::Result<(OriginalImage, ThumbnailImage)>> + Send
    where
        R: BufRead + Seek + Send + 'static;
}
