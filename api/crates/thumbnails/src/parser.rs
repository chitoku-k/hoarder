use anyhow::Context;
use domain::parser::thumbnails::{ThumbnailImageParser, ThumbnailMetadata};
use webp::Decoder;

#[derive(Clone)]
pub struct WebPImageParser;

impl ThumbnailImageParser for WebPImageParser {
    fn get_metadata(&self, body: &[u8]) -> anyhow::Result<ThumbnailMetadata> {
        let decoder = Decoder::new(body);
        let image = decoder.decode().context("failed to decode image")?;
        let metadata = ThumbnailMetadata::new(image.width(), image.height());
        Ok(metadata)
    }
}
