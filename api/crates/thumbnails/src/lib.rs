use std::sync::Arc;

use anyhow::Context;
use axum::{
    body::Body,
    extract::Path,
    http::{Response, StatusCode},
    response::IntoResponse,
    Extension,
};
use derive_more::Constructor;
use domain::{
    entity::replicas::ThumbnailId,
    service::media::MediaServiceInterface,
};

pub mod processor;

#[derive(Clone, Constructor)]
pub struct ThumbnailURLFactory {
    endpoint: String,
}

impl ThumbnailURLFactory {
    pub fn url(&self, id: &ThumbnailId) -> String {
        format!("{}{}", self.endpoint, id)
    }
}

#[derive(Clone, Constructor)]
pub struct ThumbnailsHandler<MediaService> {
    media_service: MediaService,
}

impl<MediaService> ThumbnailsHandler<MediaService>
where
    MediaService: MediaServiceInterface,
{
    async fn handle(&self, id: ThumbnailId) -> anyhow::Result<Vec<u8>> {
        let thumbnail = self.media_service.get_thumbnail_by_id(id).await.context("no thumbnail available")?;
        Ok(thumbnail)
    }
}

pub async fn handle<MediaService>(
    Extension(handler): Extension<Arc<ThumbnailsHandler<MediaService>>>,
    Path(id): Path<ThumbnailId>,
) -> impl IntoResponse
where
    MediaService: MediaServiceInterface,
{
    match handler.handle(id).await {
        Ok(thumbnail) => {
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "image/webp")
                .body(Body::from(thumbnail))
                .unwrap()
        }
        Err(_) => {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(Body::from("Not Found\n"))
                .unwrap()
        },
    }
}
