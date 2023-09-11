use std::{fs::File, io::{BufReader, Cursor}};

use anyhow::Context;
use async_trait::async_trait;
use derive_more::Constructor;
use domain::{
    entity::replicas::{Size, ThumbnailImage},
    processor::media::MediumImageProcessor,
};
use image::io::Reader;
use tokio::task;

pub use image::imageops::FilterType;
pub use image::ImageOutputFormat;

#[derive(Clone, Constructor)]
pub struct FileImageProcessor {
    thumbnail_size: Size,
    thumbnail_format: ImageOutputFormat,
    thumbnail_filter: FilterType,
}

#[async_trait]
impl MediumImageProcessor for FileImageProcessor {
    async fn generate_thumbnail(&self, path: &str) -> anyhow::Result<ThumbnailImage> {
        task::block_in_place(move || {
            let file = File::open(path).context("failed to open image")?;
            let reader = Reader::new(BufReader::new(file))
                .with_guessed_format()
                .context("failed to detect image format")?;

            let image = reader.decode().context("failed to decode image")?;
            let mut body = Vec::new();

            let thumbnail = image.resize(self.thumbnail_size.width, self.thumbnail_size.height, self.thumbnail_filter);
            thumbnail
                .write_to(&mut Cursor::new(&mut body), self.thumbnail_format.clone())
                .context("failed to generate thumbnail")?;

            Ok(ThumbnailImage::new(body, Size::new(thumbnail.width(), thumbnail.height())))
        })
    }
}
