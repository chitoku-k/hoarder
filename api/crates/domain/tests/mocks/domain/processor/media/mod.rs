use std::{future::Future, io::{BufRead, Seek}};

use domain::{
    entity::replicas::{OriginalImage, ThumbnailImage},
    error::Result,
    processor::media::MediumImageProcessor,
};

mockall::mock! {
    pub MediumImageProcessor {}

    impl MediumImageProcessor for MediumImageProcessor {
        fn generate_thumbnail<R>(&self, read: R) -> impl Future<Output = Result<(OriginalImage, ThumbnailImage)>> + Send
        where
            R: BufRead + Seek + Send + 'static;
    }

    impl Clone for MediumImageProcessor {
        fn clone(&self) -> Self;
    }
}
