use crate::{
    entity::replicas::{OriginalImage, ThumbnailImage},
    error::Result,
    io::SeekableBufRead,
};

pub trait MediumImageProcessor: Send + Sync + 'static {
    /// Generates a thumbnail for image on the given path.
    fn generate_thumbnail<R>(&self, read: R) -> Result<(OriginalImage, ThumbnailImage)>
    where
        R: SeekableBufRead;
}
