use std::future::Future;

use crate::entity::replicas::ThumbnailImage;

#[cfg_attr(feature = "test-mock", mockall::automock)]
pub trait MediumImageProcessor: Send + Sync + 'static {
    /// Generates a thumbnail for image on the given path.
    fn generate_thumbnail(&self, path: &str) -> impl Future<Output = anyhow::Result<ThumbnailImage>> + Send;
}
