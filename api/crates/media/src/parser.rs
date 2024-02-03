use std::{fs::File, io::BufReader, path::Path};

use anyhow::Context;
use derive_more::Constructor;
use domain::parser::media::{MediumImageParser, MediumMetadata};
use image::io::Reader;
use normalize_path::NormalizePath;
use tokio::task;

#[derive(Clone, Constructor)]
pub struct FileImageParser {
    root_dir: String,
}

impl MediumImageParser for FileImageParser {
    async fn get_metadata(&self, path: &str) -> anyhow::Result<MediumMetadata> {
        let path = Path::new(path).strip_prefix("/").context("absolute path required")?;
        let subpath = path.try_normalize().context("invalid path")?;
        let fullpath = Path::new(&self.root_dir).join(subpath);

        task::spawn_blocking(move || {
            let file = File::open(fullpath).context("failed to open image")?;
            let reader = Reader::new(BufReader::new(file))
                .with_guessed_format()
                .context("failed to detect image format")?;

            let format = reader.format().context("failed to get image format")?;
            let (width, height) = reader.into_dimensions().context("failed to decode image")?;
            Ok(MediumMetadata::new(format.to_mime_type(), width, height))
        }).await?
    }
}
