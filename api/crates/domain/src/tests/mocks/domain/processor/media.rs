use std::io::{BufRead, Seek};

use crate::{
    entity::replicas::{OriginalImage, ThumbnailImage},
    error::Result,
    processor::media::MediumImageProcessor,
};

mockall::mock! {
    pub(crate) MediumImageProcessor {}

    impl MediumImageProcessor for MediumImageProcessor {
        fn generate_thumbnail<R>(&self, read: R) -> Result<(OriginalImage, ThumbnailImage)>
        where
            R: BufRead + Seek + 'static;
    }

    impl Clone for MediumImageProcessor {
        fn clone(&self) -> Self;
    }
}
