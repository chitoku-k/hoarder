use std::{fs::File, io::{BufReader, Cursor}, path::Path};

use anyhow::Context;
use derive_more::Constructor;
use domain::{
    entity::replicas::{Size, ThumbnailImage},
    processor::media::MediumImageProcessor,
};
use image::io::Reader;
use normalize_path::NormalizePath;
use tokio::task;

pub use image::imageops::FilterType;
pub use image::ImageOutputFormat;

#[derive(Clone, Constructor)]
pub struct FileImageProcessor {
    root_dir: String,
    thumbnail_size: Size,
    thumbnail_format: ImageOutputFormat,
    thumbnail_filter: FilterType,
}

impl MediumImageProcessor for FileImageProcessor {
    async fn generate_thumbnail(&self, path: &str) -> anyhow::Result<ThumbnailImage> {
        let size = self.thumbnail_size;
        let filter = self.thumbnail_filter;
        let format = self.thumbnail_format.clone();

        let path = Path::new(path).strip_prefix("/").context("absolute path required")?;
        let subpath = path.try_normalize().context("invalid path")?;
        let fullpath = Path::new(&self.root_dir).join(subpath);

        task::spawn_blocking(move || {
            let file = File::open(fullpath).context("failed to open image")?;
            let reader = Reader::new(BufReader::with_capacity(5 * 1024 * 1024, file))
                .with_guessed_format()
                .context("failed to detect image format")?;

            let image = reader.decode().context("failed to decode image")?;
            let mut body = Vec::new();

            let thumbnail = image.resize(size.width, size.height, filter);
            thumbnail
                .write_to(&mut Cursor::new(&mut body), format)
                .context("failed to generate thumbnail")?;

            Ok(ThumbnailImage::new(body, Size::new(thumbnail.width(), thumbnail.height())))
        }).await?
    }
}
