use async_trait::async_trait;
use derive_more::{Constructor, From};

use crate::entity::replicas::Size;

#[cfg_attr(feature = "test-mock", mockall::automock)]
#[async_trait]
pub trait MediumImageParser: Send + Sync + 'static {
    /// Gets metadata for the image on the given path.
    async fn get_metadata(&self, path: &str) -> anyhow::Result<MediumMetadata>;
}

#[derive(Clone, Constructor, Copy, Debug, Eq, From, PartialEq)]
pub struct MediumMetadata {
    mime_type: &'static str,
    width: u32,
    height: u32,
}

impl MediumMetadata {
    pub fn mime_type(&self) -> &'static str {
        self.mime_type
    }

    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}
