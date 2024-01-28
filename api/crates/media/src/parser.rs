use std::{fs::File, io::BufReader};

use anyhow::Context;
use derive_more::Constructor;
use domain::parser::media::{MediumImageParser, MediumMetadata};
use image::io::Reader;
use tokio::task;

#[derive(Clone, Constructor)]
pub struct FileImageParser;

impl MediumImageParser for FileImageParser {
    async fn get_metadata(&self, path: &str) -> anyhow::Result<MediumMetadata> {
        let path = path.to_string();

        task::spawn_blocking(move || {
            let file = File::open(path).context("failed to open image")?;
            let reader = Reader::new(BufReader::new(file))
                .with_guessed_format()
                .context("failed to detect image format")?;

            let format = reader.format().context("failed to get image format")?;
            let (width, height) = reader.into_dimensions().context("failed to decode image")?;
            Ok(MediumMetadata::new(format.to_mime_type(), width, height))
        }).await?
    }
}
