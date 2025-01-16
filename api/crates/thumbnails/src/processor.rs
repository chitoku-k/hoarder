use std::io::{BufRead, Cursor, Seek};

use derive_more::Constructor;
use domain::{
    entity::replicas::{OriginalImage, Size, ThumbnailImage},
    error::{Error, ErrorKind, Result},
    processor::media::MediumImageProcessor,
};
use image::{DynamicImage, ImageDecoder, ImageReader};
use tokio::task;

pub use image::{imageops::FilterType, ImageFormat};

#[derive(Clone, Constructor)]
pub struct InMemoryImageProcessor {
    thumbnail_size: Size,
    thumbnail_format: ImageFormat,
    thumbnail_filter: FilterType,
}

impl MediumImageProcessor for InMemoryImageProcessor {
    async fn generate_thumbnail<R>(&self, read: R) -> Result<(OriginalImage, ThumbnailImage)>
    where
        for<'a> R: BufRead + Seek + Send + 'a,
    {
        let thumbnail_size = self.thumbnail_size;
        let thumbnail_filter = self.thumbnail_filter;
        let thumbnail_format = self.thumbnail_format;

        task::spawn_blocking(move || {
            let reader = ImageReader::new(read)
                .with_guessed_format()
                .map_err(|e| Error::new(ErrorKind::MediumReplicaReadFailed, e))?;

            let format = reader.format().ok_or(ErrorKind::MediumReplicaUnsupported)?;
            let mut decoder = reader.into_decoder()
                .map_err(|e| Error::new(ErrorKind::MediumReplicaDecodeFailed, e))?;

            let orientation = decoder.orientation()
                .map_err(|e| Error::new(ErrorKind::MediumReplicaDecodeFailed, e))?;

            let mut image = DynamicImage::from_decoder(decoder)
                .map_err(|e| Error::new(ErrorKind::MediumReplicaDecodeFailed, e))?;

            image.apply_orientation(orientation);

            let mut body = Vec::new();
            let thumbnail = image.resize(thumbnail_size.width, thumbnail_size.height, thumbnail_filter);
            thumbnail
                .write_to(&mut Cursor::new(&mut body), thumbnail_format)
                .map_err(|e| Error::new(ErrorKind::MediumReplicaEncodeFailed, e))?;

            let original_image = OriginalImage::new(format.to_mime_type(), Size::new(image.width(), image.height()));
            let thumbnail_image = ThumbnailImage::new(body, Size::new(thumbnail.width(), thumbnail.height()));
            Ok((original_image, thumbnail_image))
        }).await.map_err(Error::other)?
    }
}
