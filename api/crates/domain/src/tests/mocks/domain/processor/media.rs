use crate::{
    entity::replicas::{OriginalImage, ThumbnailImage},
    error::Result,
    io::SeekableBufRead,
    processor::media::MediumImageProcessor,
};

mockall::mock! {
    pub(crate) MediumImageProcessor {}

    impl MediumImageProcessor for MediumImageProcessor {
        #[mockall::concretize]
        fn generate_thumbnail<R>(&self, read: R) -> Result<(OriginalImage, ThumbnailImage)>
        where
            R: SeekableBufRead;
    }

    impl Clone for MediumImageProcessor {
        fn clone(&self) -> Self;
    }
}
