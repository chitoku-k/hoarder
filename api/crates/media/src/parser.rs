use std::{fs::File, io::BufReader};

use anyhow::Context;
use async_trait::async_trait;
use derive_more::Constructor;
use domain::parser::media::{MediumImageParser, MediumMetadata};
use image::io::Reader;
use tokio::task;

#[derive(Clone, Constructor)]
pub struct FileImageParser;

#[async_trait]
impl MediumImageParser for FileImageParser {
    async fn get_metadata(&self, path: &str) -> anyhow::Result<MediumMetadata> {
        task::block_in_place(move || {
            let file = File::open(path).context("failed to open image")?;
            let reader = Reader::new(BufReader::new(file))
                .with_guessed_format()
                .context("failed to detect image format")?;

            let format = reader.format().context("failed to get image format")?;
            let image = reader.decode().context("failed to decode image")?;
            Ok(MediumMetadata::new(format.to_mime_type(), image.width(), image.height()))
        })
    }
}
