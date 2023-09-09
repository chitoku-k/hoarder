use derive_more::{Constructor, From};

use crate::entity::replicas::Size;

#[cfg_attr(feature = "test-mock", mockall::automock)]
pub trait ThumbnailImageParser: Send + Sync + 'static {
    /// Gets size for the image.
    fn get_metadata(&self, body: &[u8]) -> anyhow::Result<ThumbnailMetadata>;
}

#[derive(Clone, Constructor, Copy, Debug, Eq, From, PartialEq)]
pub struct ThumbnailMetadata {
    width: u32,
    height: u32,
}

impl ThumbnailMetadata {
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}
