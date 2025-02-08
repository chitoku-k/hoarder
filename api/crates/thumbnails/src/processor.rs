use std::io::{BufRead, Cursor, Seek};

use derive_more::Constructor;
use domain::{
    entity::replicas::{OriginalImage, Size, ThumbnailImage},
    error::{Error, ErrorKind, Result},
    processor::media::MediumImageProcessor,
};
use image::{DynamicImage, ImageDecoder, ImageReader};

pub use image::{imageops::FilterType, ImageFormat};

#[derive(Clone, Constructor)]
pub struct InMemoryImageProcessor {
    thumbnail_size: Size,
    thumbnail_format: ImageFormat,
    thumbnail_filter: FilterType,
}

impl MediumImageProcessor for InMemoryImageProcessor {
    #[tracing::instrument(skip_all)]
    fn generate_thumbnail<R>(&self, read: R) -> Result<(OriginalImage, ThumbnailImage)>
    where
        R: BufRead + Seek,
    {
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
        let thumbnail = image.resize(self.thumbnail_size.width, self.thumbnail_size.height, self.thumbnail_filter);
        thumbnail
            .write_to(&mut Cursor::new(&mut body), self.thumbnail_format)
            .map_err(|e| Error::new(ErrorKind::MediumReplicaEncodeFailed, e))?;

        let original_image = OriginalImage::new(format.to_mime_type(), Size::new(image.width(), image.height()));
        let thumbnail_image = ThumbnailImage::new(body, Size::new(thumbnail.width(), thumbnail.height()));
        Ok((original_image, thumbnail_image))
    }
}
