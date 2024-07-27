use std::io::{BufRead, Cursor, Seek, SeekFrom};

use derive_more::Constructor;
use domain::{
    entity::replicas::{OriginalImage, Size, ThumbnailImage},
    error::{Error, ErrorKind, Result},
    processor::media::MediumImageProcessor,
};
use exif::{In, Tag};
use image::{imageops::{flip_horizontal, flip_vertical, rotate180, rotate270, rotate90}, DynamicImage, ImageReader};
use tokio::task;

pub use image::{imageops::FilterType, ImageFormat};

#[derive(Clone, Constructor)]
pub struct InMemoryImageProcessor {
    thumbnail_size: Size,
    thumbnail_format: ImageFormat,
    thumbnail_filter: FilterType,
}

fn rotate(image: DynamicImage, orientation: u32) -> Result<DynamicImage> {
    let image = match orientation {
        1 => image,
        2 => DynamicImage::from(flip_horizontal(&image)),
        3 => DynamicImage::from(rotate180(&image)),
        4 => DynamicImage::from(flip_vertical(&image)),
        5 => DynamicImage::from(flip_horizontal(&rotate90(&image))),
        6 => DynamicImage::from(rotate90(&image)),
        7 => DynamicImage::from(flip_horizontal(&rotate270(&image))),
        8 => DynamicImage::from(rotate270(&image)),
        _ => return Err(Error::from(ErrorKind::MediumReplicaDecodeFailed)),
    };

    Ok(image)
}

impl MediumImageProcessor for InMemoryImageProcessor {
    async fn generate_thumbnail<R>(&self, mut read: R) -> Result<(OriginalImage, ThumbnailImage)>
    where
        R: BufRead + Seek + Send + 'static,
    {
        let thumbnail_size = self.thumbnail_size;
        let thumbnail_filter = self.thumbnail_filter;
        let thumbnail_format = self.thumbnail_format;

        task::spawn_blocking(move || {
            let orientation = exif::Reader::new()
                .read_from_container(&mut read)
                .ok()
                .and_then(|e| e.get_field(Tag::Orientation, In::PRIMARY).and_then(|f| f.value.get_uint(0)))
                .unwrap_or(1);

            read.seek(SeekFrom::Start(0))
                .map_err(|e| Error::new(ErrorKind::MediumReplicaReadFailed, e))?;

            let reader = ImageReader::new(read)
                .with_guessed_format()
                .map_err(|e| Error::new(ErrorKind::MediumReplicaReadFailed, e))?;

            let format = reader.format().ok_or(ErrorKind::MediumReplicaUnsupported)?;
            let image = reader.decode()
                .map_err(|e| Error::new(ErrorKind::MediumReplicaDecodeFailed, e))
                .and_then(|i| rotate(i, orientation))?;

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
