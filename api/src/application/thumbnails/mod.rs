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

use crate::domain::{
    entity::replicas::ReplicaId,
    repository,
    service::media::MediaService,
};

#[derive(Clone, Constructor)]
pub struct ThumbnailURLFactory {
    endpoint: String,
}

impl ThumbnailURLFactory {
    pub fn url(&self, id: &ReplicaId) -> String {
        format!("{}{}", self.endpoint, id)
    }
}

#[derive(Clone, Constructor)]
pub struct ThumbnailsHandler<MediaRepository, ReplicasRepository, SourcesRepository> {
    media_service: MediaService<MediaRepository, ReplicasRepository, SourcesRepository>,
}

impl<MediaRepository, ReplicasRepository, SourcesRepository> ThumbnailsHandler<MediaRepository, ReplicasRepository, SourcesRepository>
where
    MediaRepository: repository::media::MediaRepository,
    ReplicasRepository: repository::replicas::ReplicasRepository,
    SourcesRepository: repository::sources::SourcesRepository,
{
    async fn handle(&self, id: ReplicaId) -> anyhow::Result<Vec<u8>> {
        let replica = self.media_service.get_thumbnail_by_id(id).await?;
        let thumbnail = replica.thumbnail.context("no thumbnail available")?;
        Ok(thumbnail)
    }
}

pub async fn handle<MediaRepository, ReplicasRepository, SourcesRepository>(
    Extension(handler): Extension<Arc<ThumbnailsHandler<MediaRepository, ReplicasRepository, SourcesRepository>>>,
    Path(id): Path<ReplicaId>,
) -> impl IntoResponse
where
    MediaRepository: repository::media::MediaRepository,
    ReplicasRepository: repository::replicas::ReplicasRepository,
    SourcesRepository: repository::sources::SourcesRepository,
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
