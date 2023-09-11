use async_trait::async_trait;

use crate::entity::replicas::ThumbnailImage;

#[cfg_attr(feature = "test-mock", mockall::automock)]
#[async_trait]
pub trait MediumImageProcessor: Send + Sync + 'static {
    /// Generates a thumbnail for image on the given path.
    async fn generate_thumbnail(&self, path: &str) -> anyhow::Result<ThumbnailImage>;
}
