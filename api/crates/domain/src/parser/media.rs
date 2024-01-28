use std::future::Future;

use derive_more::{Constructor, From};

use crate::entity::replicas::Size;

#[cfg_attr(feature = "test-mock", mockall::automock)]
pub trait MediumImageParser: Send + Sync + 'static {
    /// Gets metadata for the image on the given path.
    fn get_metadata(&self, path: &str) -> impl Future<Output = anyhow::Result<MediumMetadata>> + Send;
}

#[derive(Clone, Constructor, Copy, Debug, Eq, From, PartialEq)]
pub struct MediumMetadata {
    mime_type: &'static str,
    width: u32,
    height: u32,
}

impl MediumMetadata {
    pub const fn mime_type(&self) -> &'static str {
        self.mime_type
    }

    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}
