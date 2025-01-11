use std::{future::Future, io::{BufRead, Seek}};

use crate::{
    entity::replicas::{OriginalImage, ThumbnailImage},
    error::Result,
};

pub trait MediumImageProcessor: Send + Sync + 'static {
    /// Generates a thumbnail for image on the given path.
    fn generate_thumbnail<R>(&self, read: R) -> impl Future<Output = Result<(OriginalImage, ThumbnailImage)>> + Send
    where
        R: BufRead + Seek + Send + 'static;
}
